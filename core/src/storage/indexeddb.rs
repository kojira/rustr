use async_trait::async_trait;
use rexie::*;
use serde_json;
use wasm_bindgen::{JsValue, JsCast};

use crate::storage::Storage;
use crate::types::{StoredEvent, StorageFilter, DmThread, OutboxItem, OutboxStatus};
use crate::error::{Result, CoreError};

const DB_NAME: &str = "rustr_db";
const DB_VERSION: u32 = 1;

const STORE_EVENTS: &str = "events";
const STORE_DM_THREADS: &str = "dm_threads";
const STORE_LAST_SEEN: &str = "last_seen";
const STORE_OUTBOX: &str = "outbox";
const STORE_KEYPAIR: &str = "keypair";

/// IndexedDB実装
pub struct IndexedDbStorage {
    db: Rexie,
}

impl IndexedDbStorage {
    async fn open_db() -> Result<Rexie> {
        let rexie = Rexie::builder(DB_NAME)
            .version(DB_VERSION)
            .add_object_store(
                ObjectStore::new(STORE_EVENTS)
                    .key_path("id")
                    .add_index(Index::new("kind", "kind"))
                    .add_index(Index::new("pubkey", "pubkey"))
                    .add_index(Index::new("created_at", "created_at")),
            )
            .add_object_store(ObjectStore::new(STORE_DM_THREADS).key_path("peer"))
            .add_object_store(ObjectStore::new(STORE_LAST_SEEN).key_path("scope"))
            .add_object_store(
                ObjectStore::new(STORE_OUTBOX)
                    .key_path("req_id")
                    .add_index(Index::new("status", "status")),
            )
            .add_object_store(ObjectStore::new(STORE_KEYPAIR).key_path("id"))
            .build()
            .await?;

        Ok(rexie)
    }
}

#[async_trait(?Send)]
impl Storage for IndexedDbStorage {
    async fn init() -> Result<Self> {
        let db = Self::open_db().await?;
        Ok(Self { db })
    }

    async fn insert_event(&self, event: &StoredEvent) -> Result<()> {
        let tx = self.db.transaction(&[STORE_EVENTS], TransactionMode::ReadWrite)?;
        let store = tx.store(STORE_EVENTS)?;

        let js_value = serde_wasm_bindgen::to_value(event)?;

        store.put(&js_value, None).await?;
        tx.done().await?;

        Ok(())
    }

    async fn save_event(&self, event_id: &str, event_json: &str) -> Result<()> {
        let tx = self.db.transaction(&[STORE_EVENTS], TransactionMode::ReadWrite)
            .map_err(|e| {
                log::error!("save_event: Failed to start transaction for {}: {:?}", event_id, e);
                e
            })?;
        let store = tx.store(STORE_EVENTS)
            .map_err(|e| {
                log::error!("save_event: Failed to get store for {}: {:?}", event_id, e);
                e
            })?;

        // JSON文字列をパースしてStoredEvent構造体に変換
        let event: serde_json::Value = serde_json::from_str(event_json)
            .map_err(|e| {
                log::error!("save_event: Failed to parse JSON for {}: {:?}", event_id, e);
                CoreError::ParseError(e.to_string())
            })?;
        
        // StoredEvent構造体を作成（IndexedDBのkeyPathが正しく動作するように）
        let now = js_sys::Date::now() as i64;
        let stored_event = StoredEvent {
            id: event["id"].as_str().unwrap_or(event_id).to_string(),
            kind: event["kind"].as_u64().unwrap_or(0) as u16,
            pubkey: event["pubkey"].as_str().unwrap_or("").to_string(),
            created_at: event["created_at"].as_i64().unwrap_or(0),
            content: event["content"].as_str().unwrap_or("").to_string(),
            tags: event["tags"].as_array().map(|arr| {
                arr.iter().filter_map(|tag| {
                    tag.as_array().map(|t| {
                        t.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
                    })
                }).collect()
            }).unwrap_or_default(),
            sig: event["sig"].as_str().unwrap_or("").to_string(),
            relay_hint: None,
            inserted_at: now,
        };
        
        let js_value = serde_wasm_bindgen::to_value(&stored_event)
            .map_err(|e| {
                log::error!("save_event: Failed to serialize event {}: {:?}", event_id, e);
                e
            })?;
        
        store.put(&js_value, None).await
            .map_err(|e| {
                log::error!("save_event: Failed to put event {} into IndexedDB: {:?}", event_id, e);
                e
            })?;
        tx.done().await
            .map_err(|e| {
                log::error!("save_event: Failed to commit transaction for {}: {:?}", event_id, e);
                e
            })?;

        Ok(())
    }

    async fn get_events(&self, filter: &StorageFilter) -> Result<Vec<StoredEvent>> {
        let tx = self.db.transaction(&[STORE_EVENTS], TransactionMode::ReadOnly)?;
        let store = tx.store(STORE_EVENTS)?;

        // 全件取得してメモリ上でフィルタリング
        // （IndexedDBのインデックスを使った最適化は将来の改善として）
        let all = store.get_all(None, None).await?;

        let mut events = Vec::new();
        for value in all {
            // JavaScriptオブジェクトとしてデシリアライズ
            if let Ok(event) = serde_wasm_bindgen::from_value::<StoredEvent>(value) {
                // フィルター適用
                if let Some(kinds) = &filter.kinds {
                    if !kinds.contains(&event.kind) {
                        continue;
                    }
                }
                if let Some(authors) = &filter.authors {
                    if !authors.contains(&event.pubkey) {
                        continue;
                    }
                }
                if let Some(since) = filter.since {
                    if event.created_at < since {
                        continue;
                    }
                }
                if let Some(until) = filter.until {
                    if event.created_at > until {
                        continue;
                    }
                }
                
                events.push(event);
            }
        }

        // created_at降順でソート
        events.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // limit適用
        if let Some(limit) = filter.limit {
            events.truncate(limit as usize);
        }

        Ok(events)
    }

    async fn upsert_dm_thread(&self, peer: &str, last_msg_at: i64) -> Result<()> {
        let tx = self.db.transaction(&[STORE_DM_THREADS], TransactionMode::ReadWrite)?;
        let store = tx.store(STORE_DM_THREADS)?;

        let thread = DmThread {
            peer: peer.to_string(),
            last_seen: 0,
            last_msg_at,
        };

        let js_value = serde_wasm_bindgen::to_value(&thread)?;

        store.put(&js_value, None).await?;
        tx.done().await?;

        Ok(())
    }

    async fn get_dm_threads(&self) -> Result<Vec<DmThread>> {
        let tx = self.db.transaction(&[STORE_DM_THREADS], TransactionMode::ReadOnly)?;
        let store = tx.store(STORE_DM_THREADS)?;

        let all = store.get_all(None, None).await?;

        let mut threads = Vec::new();
        for value in all {
            // JavaScriptオブジェクトとしてデシリアライズ
            if let Ok(thread) = serde_wasm_bindgen::from_value::<DmThread>(value) {
                threads.push(thread);
            }
        }

        // last_msg_at降順でソート
        threads.sort_by(|a, b| b.last_msg_at.cmp(&a.last_msg_at));

        Ok(threads)
    }

    async fn get_last_seen(&self, scope: &str) -> Result<i64> {
        let tx = self.db.transaction(&[STORE_LAST_SEEN], TransactionMode::ReadOnly)?;
        let store = tx.store(STORE_LAST_SEEN)?;

        let key = JsValue::from_str(scope);
        let value = store.get(key).await?;

        if let Some(v) = value {
            // JavaScriptオブジェクトとしてデシリアライズ
            if let Ok(data) = serde_wasm_bindgen::from_value::<serde_json::Value>(v) {
                if let Some(ts) = data.get("ts").and_then(|v| v.as_i64()) {
                    return Ok(ts);
                }
            }
        }

        Ok(0)
    }

    async fn set_last_seen(&self, scope: &str, ts: i64) -> Result<()> {
        let tx = self.db.transaction(&[STORE_LAST_SEEN], TransactionMode::ReadWrite)?;
        let store = tx.store(STORE_LAST_SEEN)?;

        let data = serde_json::json!({
            "scope": scope,
            "ts": ts,
        });

        let value = serde_wasm_bindgen::to_value(&data)?;
        store.put(&value, None).await?;
        tx.done().await?;

        Ok(())
    }

    async fn enqueue_outbox(&self, item: OutboxItem) -> Result<String> {
        let req_id = item.req_id.clone();
        
        let tx = self.db.transaction(&[STORE_OUTBOX], TransactionMode::ReadWrite)
            .map_err(|e| {
                log::error!("enqueue_outbox: Failed to start transaction for {}: {:?}", req_id, e);
                e
            })?;
        let store = tx.store(STORE_OUTBOX)
            .map_err(|e| {
                log::error!("enqueue_outbox: Failed to get store for {}: {:?}", req_id, e);
                e
            })?;
        
        // serde_wasm_bindgenを使ってJavaScriptオブジェクトに変換
        let js_value = serde_wasm_bindgen::to_value(&item)
            .map_err(|e| {
                log::error!("enqueue_outbox: Failed to serialize item {}: {:?}", req_id, e);
                e
            })?;
        
        // key_path("req_id")が設定されているので、キーはオブジェクトから自動取得される
        store.put(&js_value, None).await
            .map_err(|e| {
                log::error!("enqueue_outbox: Failed to put item {}: {:?}", req_id, e);
                e
            })?;
        tx.done().await
            .map_err(|e| {
                log::error!("enqueue_outbox: Failed to commit transaction for {}: {:?}", req_id, e);
                e
            })?;

        Ok(req_id)
    }

    async fn update_outbox_status(&self, req_id: &str, status: OutboxStatus) -> Result<()> {
        let tx = self.db.transaction(&[STORE_OUTBOX], TransactionMode::ReadWrite)
            .map_err(|e| {
                log::error!("update_outbox_status: Failed to start transaction for {}: {:?}", req_id, e);
                e
            })?;
        let store = tx.store(STORE_OUTBOX)
            .map_err(|e| {
                log::error!("update_outbox_status: Failed to get store for {}: {:?}", req_id, e);
                e
            })?;

        let key = JsValue::from_str(req_id);
        let value = store.get(key.clone()).await
            .map_err(|e| {
                log::error!("update_outbox_status: Failed to get item {}: {:?}", req_id, e);
                e
            })?;

        if let Some(v) = value {
            // JavaScriptオブジェクトとしてデシリアライズ
            if let Ok(mut item) = serde_wasm_bindgen::from_value::<OutboxItem>(v.clone()) {
                item.status = status;
                let js_value = serde_wasm_bindgen::to_value(&item)
                    .map_err(|e| {
                        log::error!("update_outbox_status: Failed to serialize updated item {}: {:?}", req_id, e);
                        e
                    })?;
                // key_path("req_id")が設定されているので、キーはオブジェクトから自動取得される
                store.put(&js_value, None).await
                    .map_err(|e| {
                        log::error!("update_outbox_status: Failed to put updated item {}: {:?}", req_id, e);
                        e
                    })?;
            } else {
                // 古い形式のデータは削除
                log::warn!("update_outbox_status: Deleting old format outbox item: {}, value type: {:?}", req_id, v);
                let delete_key = JsValue::from_str(req_id);
                store.delete(delete_key).await
                    .map_err(|e| {
                        log::error!("update_outbox_status: Failed to delete old format item {}: {:?}", req_id, e);
                        e
                    })?;
            }
        } else {
            log::warn!("update_outbox_status: Item not found: {}", req_id);
        }

        tx.done().await
            .map_err(|e| {
                log::error!("update_outbox_status: Failed to commit transaction for {}: {:?}", req_id, e);
                e
            })?;
        Ok(())
    }

    async fn get_pending_outbox(&self) -> Result<Vec<OutboxItem>> {
        let tx = self.db.transaction(&[STORE_OUTBOX], TransactionMode::ReadOnly)
            .map_err(|e| {
                log::error!("get_pending_outbox: Failed to start transaction: {:?}", e);
                e
            })?;
        let store = tx.store(STORE_OUTBOX)
            .map_err(|e| {
                log::error!("get_pending_outbox: Failed to get store: {:?}", e);
                e
            })?;

        let all = store.get_all(None, None).await
            .map_err(|e| {
                log::error!("get_pending_outbox: Failed to get all items: {:?}", e);
                e
            })?;

        let mut items = Vec::new();
        for (idx, value) in all.iter().enumerate() {
            // JavaScriptオブジェクトとしてデシリアライズ
            if let Ok(item) = serde_wasm_bindgen::from_value::<OutboxItem>(value.clone()) {
                if matches!(item.status, OutboxStatus::Queued | OutboxStatus::Sent) {
                    items.push(item);
                }
            } else {
                // 古い形式のデータ（JSON文字列）は警告してスキップ
                log::warn!("get_pending_outbox: Skipping old format outbox item at index {}, value type: {:?}", idx, value);
            }
        }

        Ok(items)
    }

    async fn save_keypair(&self, encrypted_data: &[u8]) -> Result<()> {
        let tx = self.db.transaction(&[STORE_KEYPAIR], TransactionMode::ReadWrite)?;
        let store = tx.store(STORE_KEYPAIR)?;

        let data = serde_json::json!({
            "id": "default",
            "data": base64_encode(encrypted_data),
        });

        let value = serde_wasm_bindgen::to_value(&data)?;
        store.put(&value, None).await?;
        tx.done().await?;

        Ok(())
    }

    async fn get_keypair(&self) -> Result<Option<Vec<u8>>> {
        let tx = self.db.transaction(&[STORE_KEYPAIR], TransactionMode::ReadOnly)?;
        let store = tx.store(STORE_KEYPAIR)?;

        let key = JsValue::from_str("default");
        let value = store.get(key).await?;

        if let Some(v) = value {
            // JavaScriptオブジェクトとしてデシリアライズ
            if let Ok(data) = serde_wasm_bindgen::from_value::<serde_json::Value>(v) {
                if let Some(encoded) = data.get("data").and_then(|v| v.as_str()) {
                    if let Ok(decoded) = base64_decode(encoded) {
                        return Ok(Some(decoded));
                    }
                }
            }
        }

        Ok(None)
    }
}

// 簡易Base64エンコード/デコード
fn base64_encode(data: &[u8]) -> String {
    let window = web_sys::window().unwrap();
    let btoa = js_sys::Reflect::get(&window, &JsValue::from_str("btoa")).unwrap();
    let func = btoa.unchecked_ref::<js_sys::Function>();
    
    // バイト配列を文字列に変換
    let binary_string: String = data.iter().map(|&b| b as char).collect();
    let result = func.call1(&window, &JsValue::from_str(&binary_string)).unwrap();
    result.as_string().unwrap()
}

fn base64_decode(encoded: &str) -> Result<Vec<u8>> {
    let window = web_sys::window().ok_or_else(|| CoreError::Other("No window".to_string()))?;
    let atob = js_sys::Reflect::get(&window, &JsValue::from_str("atob"))?;
    let func = atob.unchecked_ref::<js_sys::Function>();
    
    let result = func.call1(&window, &JsValue::from_str(encoded))?;
    let decoded_str = result.as_string().ok_or_else(|| CoreError::Other("atob result not a string".to_string()))?;
    
    Ok(decoded_str.bytes().collect())
}

