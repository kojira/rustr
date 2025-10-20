use std::collections::VecDeque;
use std::sync::Arc;

use crate::storage::Storage;
use crate::types::{OutboxItem, OutboxStatus};
use crate::relay::RelayConnection;
use crate::error::Result;

const MAX_RETRY_COUNT: u32 = 5;
const RETRY_DELAY_SECONDS: i64 = 5;

/// 送信キュー
pub struct OutboxQueue {
    storage: Arc<dyn Storage>,
    pending: VecDeque<OutboxItem>,
}

impl OutboxQueue {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            storage,
            pending: VecDeque::new(),
        }
    }

    /// イベントをキューに追加
    pub async fn enqueue(&mut self, event_json: String) -> Result<String> {
        let req_id = generate_req_id();
        let now = current_timestamp();

        let item = OutboxItem {
            req_id: req_id.clone(),
            event_json,
            status: OutboxStatus::Queued,
            last_try_at: now,
            retry_count: 0,
            error: None,
        };

        self.storage.enqueue_outbox(item.clone()).await?;
        self.pending.push_back(item);

        Ok(req_id)
    }

    /// 保留中のアイテムをStorageから読み込み
    pub async fn load_pending(&mut self) -> Result<()> {
        let items = self.storage.get_pending_outbox().await?;
        self.pending.extend(items);
        Ok(())
    }

    /// キューから1つ取り出す（送信用）
    pub async fn dequeue(&mut self) -> Result<Option<String>> {
        if let Some(item) = self.pending.front() {
            if item.status == OutboxStatus::Queued {
                return Ok(Some(item.event_json.clone()));
            }
        }
        Ok(None)
    }

    /// キューを処理（Relayに送信）
    pub async fn process(&mut self, relays: &[RelayConnection]) -> Result<()> {
        let now = current_timestamp();
        let mut to_retry = Vec::new();

        while let Some(mut item) = self.pending.pop_front() {
            // 再送待ち時間チェック
            if item.status == OutboxStatus::Sent {
                let elapsed = now - item.last_try_at;
                if elapsed < RETRY_DELAY_SECONDS {
                    to_retry.push(item);
                    continue;
                }
            }

            // 最大再送回数チェック
            if item.retry_count >= MAX_RETRY_COUNT {
                item.status = OutboxStatus::Error;
                item.error = Some("Max retry count exceeded".to_string());
                self.storage.update_outbox_status(&item.req_id, OutboxStatus::Error).await?;
                continue;
            }

            // 接続済みのRelayに送信
            let mut sent_count = 0;
            for relay in relays {
                if relay.is_connected() {
                    if let Err(e) = relay.send(&item.event_json).await {
                        log::warn!("Failed to send to {}: {}", relay.url, e);
                    } else {
                        sent_count += 1;
                    }
                }
            }

            if sent_count > 0 {
                item.status = OutboxStatus::Sent;
                item.last_try_at = now;
                item.retry_count += 1;
                self.storage.update_outbox_status(&item.req_id, OutboxStatus::Sent).await?;
                to_retry.push(item);
            } else {
                // 接続済みのRelayがない場合は再度キューに戻す
                to_retry.push(item);
            }
        }

        // 再送待ちのアイテムをキューに戻す
        self.pending.extend(to_retry);

        Ok(())
    }

    /// NIP-20 OK受信時の処理
    pub async fn on_ok(&mut self, event_id: &str, accepted: bool, message: &str) -> Result<()> {
        // event_idからreq_idを探す（簡易実装: event_jsonをパースして確認）
        let mut found_req_id = None;
        
        for item in &self.pending {
            if item.event_json.contains(event_id) {
                found_req_id = Some(item.req_id.clone());
                break;
            }
        }

        if let Some(req_id) = found_req_id {
            if accepted {
                // 成功: キューから削除
                self.pending.retain(|item| item.req_id != req_id);
                self.storage.update_outbox_status(&req_id, OutboxStatus::Ok).await?;
                log::info!("Event {} accepted", event_id);
            } else {
                // 拒否: エラーとしてマーク
                if let Some(item) = self.pending.iter_mut().find(|i| i.req_id == req_id) {
                    item.status = OutboxStatus::Error;
                    item.error = Some(message.to_string());
                }
                self.storage.update_outbox_status(&req_id, OutboxStatus::Error).await?;
                log::warn!("Event {} rejected: {}", event_id, message);
            }
        }

        Ok(())
    }

    /// 失敗したアイテムを再送
    pub async fn retry_failed(&mut self) -> Result<()> {
        let now = current_timestamp();

        for item in &mut self.pending {
            if item.status == OutboxStatus::Error && item.retry_count < MAX_RETRY_COUNT {
                let elapsed = now - item.last_try_at;
                if elapsed >= RETRY_DELAY_SECONDS * (item.retry_count as i64) {
                    item.status = OutboxStatus::Queued;
                    item.error = None;
                }
            }
        }

        Ok(())
    }

    /// キューのサイズ
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// キューが空か
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

/// リクエストID生成
fn generate_req_id() -> String {
    use js_sys::Math;
    let random = Math::random();
    let timestamp = js_sys::Date::now();
    format!("req_{}_{}", timestamp as u64, (random * 1000000.0) as u64)
}

/// 現在のUNIXタイムスタンプ（秒）
fn current_timestamp() -> i64 {
    (js_sys::Date::now() / 1000.0) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::mock::MockStorage;

    #[tokio::test]
    async fn test_enqueue() {
        let storage = Arc::new(MockStorage::new());
        let mut queue = OutboxQueue::new(storage);

        let event_json = r#"{"id":"test","kind":1}"#.to_string();
        let req_id = queue.enqueue(event_json).await.unwrap();

        assert!(!req_id.is_empty());
        assert_eq!(queue.len(), 1);
    }

    #[tokio::test]
    async fn test_on_ok_accepted() {
        let storage = Arc::new(MockStorage::new());
        let mut queue = OutboxQueue::new(storage);

        let event_json = r#"{"id":"event123","kind":1}"#.to_string();
        queue.enqueue(event_json).await.unwrap();

        queue.on_ok("event123", true, "").await.unwrap();
        assert_eq!(queue.len(), 0);
    }

    #[tokio::test]
    async fn test_on_ok_rejected() {
        let storage = Arc::new(MockStorage::new());
        let mut queue = OutboxQueue::new(storage);

        let event_json = r#"{"id":"event123","kind":1}"#.to_string();
        queue.enqueue(event_json).await.unwrap();

        queue.on_ok("event123", false, "duplicate").await.unwrap();
        
        // エラーステータスになっているが、キューには残っている
        assert_eq!(queue.len(), 1);
        let item = &queue.pending[0];
        assert_eq!(item.status, OutboxStatus::Error);
        assert!(item.error.is_some());
    }
}

