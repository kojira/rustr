use async_trait::async_trait;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{CryptoKey, SubtleCrypto};
use js_sys::{Uint8Array, Object, Reflect};
use nostr::{Keys, Event, EventBuilder, Kind, Tag, Timestamp};

use super::{Signer, UnsignedEvent, SignedEvent};
use crate::storage::Storage;
use crate::error::{Result, CoreError};

/// 内蔵Signer（WebCrypto + IndexedDB）
pub struct InternalSigner {
    keys: Keys,
}

impl InternalSigner {
    /// 新規生成
    pub async fn generate(_passphrase: &str) -> Result<Self> {
        let keys = Keys::generate();
        Ok(Self { keys })
    }
    
    /// 秘密鍵から復元
    pub fn from_secret_key(secret_key: &[u8]) -> Result<Self> {
        let secret_key_hex = hex::encode(secret_key);
        let keys = Keys::parse(&secret_key_hex)
            .map_err(|e| CoreError::SignerError(format!("Invalid secret key: {}", e)))?;
        Ok(Self { keys })
    }

    /// Storageから読み込み
    pub async fn load_from_storage(passphrase: &str, storage: &dyn Storage) -> Result<Self> {
        let encrypted_data = storage.get_keypair().await?
            .ok_or_else(|| CoreError::StorageError("No keypair found in storage".to_string()))?;
        
        let decrypted = decrypt_with_passphrase(&encrypted_data, passphrase).await?;
        
        Self::from_secret_key(&decrypted)
    }

    /// Storageに保存
    pub async fn save_to_storage(&self, passphrase: &str, storage: &dyn Storage) -> Result<()> {
        let secret_key = self.keys.secret_key();
        let secret_bytes = secret_key.to_secret_bytes();
        let encrypted = encrypt_with_passphrase(&secret_bytes, passphrase).await?;
        storage.save_keypair(&encrypted).await?;
        Ok(())
    }
}

#[async_trait(?Send)]
impl Signer for InternalSigner {
    async fn get_public_key(&self) -> Result<String> {
        Ok(self.keys.public_key().to_hex())
    }

    async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<SignedEvent> {
        // Tagsを変換
        let tags: Vec<Tag> = unsigned.tags.iter()
            .filter_map(|tag_vec| Tag::parse(tag_vec).ok())
            .collect();
        
        let kind = Kind::from(unsigned.kind);
        let event = EventBuilder::new(kind, unsigned.content)
            .tags(tags)
            .sign(&self.keys)
            .await
            .map_err(|e| CoreError::SignerError(format!("Failed to sign event: {}", e)))?;
        
        Ok(SignedEvent {
            id: event.id.to_hex(),
            pubkey: event.pubkey.to_hex(),
            created_at: event.created_at.as_u64() as i64,
            kind: event.kind.as_u16(),
            tags: event.tags.iter().map(|t| {
                let vec = t.clone().to_vec();
                vec.iter().map(|s| s.to_string()).collect()
            }).collect(),
            content: event.content,
            sig: event.sig.to_string(),
        })
    }

    async fn nip04_encrypt(&self, pubkey: &str, plaintext: &str) -> Result<String> {
        let public_key = nostr::PublicKey::from_hex(pubkey)
            .map_err(|e| CoreError::SignerError(format!("Invalid pubkey: {}", e)))?;
        
        let encrypted = nostr::nips::nip04::encrypt(
            self.keys.secret_key(),
            &public_key,
            plaintext
        ).map_err(|e| CoreError::SignerError(format!("NIP-04 encryption failed: {}", e)))?;
        
        Ok(encrypted)
    }

    async fn nip04_decrypt(&self, pubkey: &str, ciphertext: &str) -> Result<String> {
        let public_key = nostr::PublicKey::from_hex(pubkey)
            .map_err(|e| CoreError::SignerError(format!("Invalid pubkey: {}", e)))?;
        
        let decrypted = nostr::nips::nip04::decrypt(
            self.keys.secret_key(),
            &public_key,
            ciphertext
        ).map_err(|e| CoreError::SignerError(format!("NIP-04 decryption failed: {}", e)))?;
        
        Ok(decrypted)
    }
}

/// パスフレーズで暗号化
async fn encrypt_with_passphrase(data: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let crypto = get_subtle_crypto()?;
    
    // PBKDF2でパスフレーズから鍵を導出
    let salt = b"rustr_salt"; // 本来はランダムに生成して保存すべき
    let key = derive_key(&crypto, passphrase, salt).await?;
    
    // AES-GCMで暗号化
    let iv = b"rustr_iv_12b"; // 本来はランダムに生成
    let encrypted = aes_gcm_encrypt(&crypto, &key, iv, data).await?;
    
    Ok(encrypted)
}

/// パスフレーズで復号化
async fn decrypt_with_passphrase(encrypted: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let crypto = get_subtle_crypto()?;
    
    let salt = b"rustr_salt";
    let key = derive_key(&crypto, passphrase, salt).await?;
    
    let iv = b"rustr_iv_12b";
    let decrypted = aes_gcm_decrypt(&crypto, &key, iv, encrypted).await?;
    
    Ok(decrypted)
}

/// SubtleCryptoを取得
fn get_subtle_crypto() -> Result<SubtleCrypto> {
    let window = web_sys::window().ok_or_else(|| CoreError::Other("No window".to_string()))?;
    let crypto = window.crypto().map_err(|_| CoreError::Other("No crypto".to_string()))?;
    Ok(crypto.subtle())
}

/// PBKDF2で鍵を導出
async fn derive_key(crypto: &SubtleCrypto, passphrase: &str, salt: &[u8]) -> Result<CryptoKey> {
    // パスフレーズをインポート
    let passphrase_bytes = passphrase.as_bytes();
    let passphrase_array = Uint8Array::from(passphrase_bytes);
    
    let import_params = Object::new();
    Reflect::set(&import_params, &"name".into(), &"PBKDF2".into())?;
    
    let base_key = JsFuture::from(
        crypto.import_key_with_object(
            "raw",
            &passphrase_array,
            &import_params,
            false,
            &js_sys::Array::of1(&"deriveBits".into()),
        )?
    ).await?;
    
    let base_key = base_key.dyn_into::<CryptoKey>()?;
    
    // PBKDF2パラメータ
    let derive_params = Object::new();
    Reflect::set(&derive_params, &"name".into(), &"PBKDF2".into())?;
    Reflect::set(&derive_params, &"salt".into(), &Uint8Array::from(salt))?;
    Reflect::set(&derive_params, &"iterations".into(), &100000.into())?;
    Reflect::set(&derive_params, &"hash".into(), &"SHA-256".into())?;
    
    let aes_params = Object::new();
    Reflect::set(&aes_params, &"name".into(), &"AES-GCM".into())?;
    Reflect::set(&aes_params, &"length".into(), &256.into())?;
    
    let usages = js_sys::Array::of2(&"encrypt".into(), &"decrypt".into());
    let derived_key = JsFuture::from(
        crypto.derive_key_with_object_and_object(
            &derive_params,
            &base_key,
            &aes_params,
            false,
            &usages,
        )?
    ).await?;
    
    Ok(derived_key.dyn_into::<CryptoKey>()?)
}

/// AES-GCM暗号化
async fn aes_gcm_encrypt(crypto: &SubtleCrypto, key: &CryptoKey, iv: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let params = Object::new();
    Reflect::set(&params, &"name".into(), &"AES-GCM".into())?;
    Reflect::set(&params, &"iv".into(), &Uint8Array::from(iv))?;
    
    let encrypted = JsFuture::from(
        crypto.encrypt_with_object_and_u8_array(&params, key, data)?
    ).await?;
    
    let encrypted_array = Uint8Array::new(&encrypted);
    Ok(encrypted_array.to_vec())
}

/// AES-GCM復号化
async fn aes_gcm_decrypt(crypto: &SubtleCrypto, key: &CryptoKey, iv: &[u8], encrypted: &[u8]) -> Result<Vec<u8>> {
    let params = Object::new();
    Reflect::set(&params, &"name".into(), &"AES-GCM".into())?;
    Reflect::set(&params, &"iv".into(), &Uint8Array::from(iv))?;
    
    let decrypted = JsFuture::from(
        crypto.decrypt_with_object_and_u8_array(&params, key, encrypted)?
    ).await?;
    
    let decrypted_array = Uint8Array::new(&decrypted);
    Ok(decrypted_array.to_vec())
}

