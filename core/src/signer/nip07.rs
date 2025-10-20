use async_trait::async_trait;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Object, Reflect};
use serde_json;

use super::{Signer, UnsignedEvent, SignedEvent};
use crate::error::{Result, CoreError};

/// NIP-07 Signer (window.nostr)
pub struct Nip07Signer;

impl Nip07Signer {
    /// NIP-07が利用可能か
    pub fn is_available() -> bool {
        if let Some(window) = web_sys::window() {
            if let Ok(nostr) = Reflect::get(&window, &JsValue::from_str("nostr")) {
                return !nostr.is_undefined()
            }
        }
        false
    }

    /// window.nostrオブジェクトを取得
    fn get_nostr() -> Result<Object> {
        let window = web_sys::window().ok_or_else(|| CoreError::Other("No window object".to_string()))?;
        let nostr = Reflect::get(&window, &JsValue::from_str("nostr"))?;
        
        if nostr.is_undefined() {
            return Err(CoreError::SignerError("window.nostr is undefined".to_string()));
        }

        Ok(nostr.into())
    }

    /// メソッド呼び出し
    async fn call_method(&self, method: &str, args: &[JsValue]) -> Result<JsValue> {
        let nostr = Self::get_nostr()?;
        let func = Reflect::get(&nostr, &JsValue::from_str(method))?;
        
        let func = func.unchecked_ref::<js_sys::Function>();

        let promise = match args.len() {
            0 => func.call0(&nostr)?,
            1 => func.call1(&nostr, &args[0])?,
            2 => func.call2(&nostr, &args[0], &args[1])?,
            _ => return Err(CoreError::Other("Too many arguments".to_string())),
        };

        let result = JsFuture::from(js_sys::Promise::from(promise)).await?;
        Ok(result)
    }
}

#[async_trait(?Send)]
impl Signer for Nip07Signer {
    async fn get_public_key(&self) -> Result<String> {
        let result = self.call_method("getPublicKey", &[]).await?;
        result.as_string().ok_or_else(|| CoreError::SignerError("Public key is not a string".to_string()))
    }

    async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<SignedEvent> {
        let event_obj = js_sys::Object::new();
        
        Reflect::set(&event_obj, &"kind".into(), &JsValue::from_f64(unsigned.kind as f64))?;
        Reflect::set(&event_obj, &"content".into(), &JsValue::from_str(&unsigned.content))?;
        Reflect::set(&event_obj, &"created_at".into(), &JsValue::from_f64(unsigned.created_at as f64))?;
        
        // tagsを変換
        let tags_array = js_sys::Array::new();
        for tag in &unsigned.tags {
            let tag_array = js_sys::Array::new();
            for item in tag {
                tag_array.push(&JsValue::from_str(item));
            }
            tags_array.push(&tag_array);
        }
        Reflect::set(&event_obj, &"tags".into(), &tags_array)?;

        let result = self.call_method("signEvent", &[event_obj.into()]).await?;
        
        // 結果をパース
        let json_str = js_sys::JSON::stringify(&result)?
            .as_string()
            .ok_or_else(|| CoreError::SignerError("Failed to stringify result".to_string()))?;
        
        let event: serde_json::Value = serde_json::from_str(&json_str)?;
        
        Ok(SignedEvent {
            id: event["id"].as_str().ok_or_else(|| CoreError::ParseError("No id".to_string()))?.to_string(),
            pubkey: event["pubkey"].as_str().ok_or_else(|| CoreError::ParseError("No pubkey".to_string()))?.to_string(),
            created_at: event["created_at"].as_i64().ok_or_else(|| CoreError::ParseError("No created_at".to_string()))?,
            kind: event["kind"].as_u64().ok_or_else(|| CoreError::ParseError("No kind".to_string()))? as u16,
            tags: serde_json::from_value(event["tags"].clone())?,
            content: event["content"].as_str().ok_or_else(|| CoreError::ParseError("No content".to_string()))?.to_string(),
            sig: event["sig"].as_str().ok_or_else(|| CoreError::ParseError("No sig".to_string()))?.to_string(),
        })
    }

    async fn nip04_encrypt(&self, _pubkey: &str, _plaintext: &str) -> Result<String> {
        // TODO: NIP-04暗号化実装
        Err(CoreError::Other("NIP-04 encrypt not implemented".to_string()))
    }

    async fn nip04_decrypt(&self, _pubkey: &str, _ciphertext: &str) -> Result<String> {
        // TODO: NIP-04復号化実装
        Err(CoreError::Other("NIP-04 decrypt not implemented".to_string()))
    }
}

