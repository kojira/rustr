pub mod nip07;
pub mod internal;

use async_trait::async_trait;
use crate::error::Result;

/// 署名者の抽象trait
/// WASM環境ではシングルスレッドのため、Send + Sync要件なし
#[async_trait(?Send)]
pub trait Signer {
    /// 公開鍵を取得
    async fn get_public_key(&self) -> Result<String>;

    /// イベントに署名
    async fn sign_event(&self, unsigned_event: UnsignedEvent) -> Result<SignedEvent>;

    /// NIP-04暗号化
    async fn nip04_encrypt(&self, pubkey: &str, plaintext: &str) -> Result<String>;

    /// NIP-04復号化
    async fn nip04_decrypt(&self, pubkey: &str, ciphertext: &str) -> Result<String>;
}

/// 未署名イベント
#[derive(Debug, Clone)]
pub struct UnsignedEvent {
    pub kind: u16,
    pub content: String,
    pub tags: Vec<Vec<String>>,
    pub created_at: i64,
}

/// 署名済みイベント
#[derive(Debug, Clone)]
pub struct SignedEvent {
    pub id: String,
    pub pubkey: String,
    pub created_at: i64,
    pub kind: u16,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

impl SignedEvent {
    /// JSONに変換
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "id": self.id,
            "pubkey": self.pubkey,
            "created_at": self.created_at,
            "kind": self.kind,
            "tags": self.tags,
            "content": self.content,
            "sig": self.sig,
        })
        .to_string()
    }
}

