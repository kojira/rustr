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

        let value = serde_json::to_string(event)?;
        let js_value = JsValue::from_str(&value);

        store.put(&js_value, None).await?;
        tx.done().await?;

        Ok(())
    }

    async fn save_event(&self, _event_id: &str, event_json: &str) -> Result<()> {
        let tx = self.db.transaction(&[STORE_EVENTS], TransactionMode::ReadWrite)?;
        let store = tx.store(STORE_EVENTS)?;

        let js_value = JsValue::from_str(event_json);
        store.put(&js_value, None).await?;
        tx.done().await?;

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
            if let Some(json_str) = value.as_string() {
                if let Ok(event) = serde_json::from_str::<StoredEvent>(&json_str) {
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

        let value = serde_json::to_string(&thread)?;
        let js_value = JsValue::from_str(&value);

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
            if let Some(json_str) = value.as_string() {
                if let Ok(thread) = serde_json::from_str::<DmThread>(&json_str) {
                    threads.push(thread);
                }
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
            if let Some(json_str) = v.as_string() {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(ts) = data.get("ts").and_then(|v| v.as_i64()) {
                        return Ok(ts);
                    }
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

        let value = JsValue::from_str(&data.to_string());
        store.put(&value, None).await?;
        tx.done().await?;

        Ok(())
    }

    async fn enqueue_outbox(&self, item: OutboxItem) -> Result<String> {
        let tx = self.db.transaction(&[STORE_OUTBOX], TransactionMode::ReadWrite)?;
        let store = tx.store(STORE_OUTBOX)?;

        let req_id = item.req_id.clone();
        let value = serde_json::to_string(&item)?;
        let js_value = JsValue::from_str(&value);

        store.put(&js_value, None).await?;
        tx.done().await?;

        Ok(req_id)
    }

    async fn update_outbox_status(&self, req_id: &str, status: OutboxStatus) -> Result<()> {
        let tx = self.db.transaction(&[STORE_OUTBOX], TransactionMode::ReadWrite)?;
        let store = tx.store(STORE_OUTBOX)?;

        let key = JsValue::from_str(req_id);
        let value = store.get(key).await?;

        if let Some(v) = value {
            if let Some(json_str) = v.as_string() {
                if let Ok(mut item) = serde_json::from_str::<OutboxItem>(&json_str) {
                    item.status = status;
                    let updated = serde_json::to_string(&item)?;
                    let js_value = JsValue::from_str(&updated);
                    store.put(&js_value, None).await?;
                }
            }
        }

        tx.done().await?;
        Ok(())
    }

    async fn get_pending_outbox(&self) -> Result<Vec<OutboxItem>> {
        let tx = self.db.transaction(&[STORE_OUTBOX], TransactionMode::ReadOnly)?;
        let store = tx.store(STORE_OUTBOX)?;

        let all = store.get_all(None, None).await?;

        let mut items = Vec::new();
        for value in all {
            if let Some(json_str) = value.as_string() {
                if let Ok(item) = serde_json::from_str::<OutboxItem>(&json_str) {
                    if matches!(item.status, OutboxStatus::Queued | OutboxStatus::Sent) {
                        items.push(item);
                    }
                }
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

        let value = JsValue::from_str(&data.to_string());
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
            if let Some(json_str) = v.as_string() {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(encoded) = data.get("data").and_then(|v| v.as_str()) {
                        if let Ok(decoded) = base64_decode(encoded) {
                            return Ok(Some(decoded));
                        }
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

