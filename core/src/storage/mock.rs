use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::storage::Storage;
use crate::types::{StoredEvent, StorageFilter, DmThread, OutboxItem, OutboxStatus};
use crate::error::Result;

/// テスト用のモックStorage実装
#[derive(Clone)]
pub struct MockStorage {
    events: Arc<Mutex<Vec<StoredEvent>>>,
    dm_threads: Arc<Mutex<Vec<DmThread>>>,
    last_seen: Arc<Mutex<HashMap<String, i64>>>,
    outbox: Arc<Mutex<Vec<OutboxItem>>>,
    keypair: Arc<Mutex<Option<Vec<u8>>>>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            dm_threads: Arc::new(Mutex::new(Vec::new())),
            last_seen: Arc::new(Mutex::new(HashMap::new())),
            outbox: Arc::new(Mutex::new(Vec::new())),
            keypair: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait(?Send)]
impl Storage for MockStorage {
    async fn init() -> Result<Self> {
        Ok(Self::new())
    }

    async fn insert_event(&self, event: &StoredEvent) -> Result<()> {
        let mut events = self.events.lock().unwrap();
        events.push(event.clone());
        Ok(())
    }

    async fn save_event(&self, _event_id: &str, _event_json: &str) -> Result<()> {
        // Mock implementation: do nothing
        Ok(())
    }

    async fn get_events(&self, filter: &StorageFilter) -> Result<Vec<StoredEvent>> {
        let events = self.events.lock().unwrap();
        let mut result: Vec<_> = events.iter().cloned().collect();

        // フィルター適用
        if let Some(kinds) = &filter.kinds {
            result.retain(|e| kinds.contains(&e.kind));
        }
        if let Some(authors) = &filter.authors {
            result.retain(|e| authors.contains(&e.pubkey));
        }
        if let Some(since) = filter.since {
            result.retain(|e| e.created_at >= since);
        }
        if let Some(until) = filter.until {
            result.retain(|e| e.created_at <= until);
        }

        // created_at降順でソート
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // limit適用
        if let Some(limit) = filter.limit {
            result.truncate(limit as usize);
        }

        Ok(result)
    }

    async fn upsert_dm_thread(&self, peer: &str, last_msg_at: i64) -> Result<()> {
        let mut threads = self.dm_threads.lock().unwrap();
        if let Some(thread) = threads.iter_mut().find(|t| t.peer == peer) {
            thread.last_msg_at = last_msg_at;
        } else {
            threads.push(DmThread {
                peer: peer.to_string(),
                last_seen: 0,
                last_msg_at,
            });
        }
        Ok(())
    }

    async fn get_dm_threads(&self) -> Result<Vec<DmThread>> {
        let threads = self.dm_threads.lock().unwrap();
        let mut result = threads.clone();
        result.sort_by(|a, b| b.last_msg_at.cmp(&a.last_msg_at));
        Ok(result)
    }

    async fn get_last_seen(&self, scope: &str) -> Result<i64> {
        let last_seen = self.last_seen.lock().unwrap();
        Ok(*last_seen.get(scope).unwrap_or(&0))
    }

    async fn set_last_seen(&self, scope: &str, ts: i64) -> Result<()> {
        let mut last_seen = self.last_seen.lock().unwrap();
        last_seen.insert(scope.to_string(), ts);
        Ok(())
    }

    async fn enqueue_outbox(&self, item: OutboxItem) -> Result<String> {
        let mut outbox = self.outbox.lock().unwrap();
        let req_id = item.req_id.clone();
        outbox.push(item);
        Ok(req_id)
    }

    async fn update_outbox_status(&self, req_id: &str, status: OutboxStatus) -> Result<()> {
        let mut outbox = self.outbox.lock().unwrap();
        if let Some(item) = outbox.iter_mut().find(|i| i.req_id == req_id) {
            item.status = status;
        }
        Ok(())
    }

    async fn get_pending_outbox(&self) -> Result<Vec<OutboxItem>> {
        let outbox = self.outbox.lock().unwrap();
        Ok(outbox
            .iter()
            .filter(|i| matches!(i.status, OutboxStatus::Queued | OutboxStatus::Sent))
            .cloned()
            .collect())
    }

    async fn save_keypair(&self, encrypted_data: &[u8]) -> Result<()> {
        let mut keypair = self.keypair.lock().unwrap();
        *keypair = Some(encrypted_data.to_vec());
        Ok(())
    }

    async fn get_keypair(&self) -> Result<Option<Vec<u8>>> {
        let keypair = self.keypair.lock().unwrap();
        Ok(keypair.clone())
    }
}

