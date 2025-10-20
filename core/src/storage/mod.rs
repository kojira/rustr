pub mod indexeddb;
pub mod mock;

use async_trait::async_trait;
use crate::error::Result;
use crate::types::{StoredEvent, StorageFilter, DmThread, OutboxItem, OutboxStatus};

/// Storage抽象trait
/// 
/// 将来的にネイティブ実装（SQLite等）への移行を想定した抽象インターフェース
/// WASM環境ではシングルスレッドのため、Send + Sync要件なし
#[async_trait(?Send)]
pub trait Storage {
    /// 初期化
    async fn init() -> Result<Self>
    where
        Self: Sized;

    /// イベント挿入
    async fn insert_event(&self, event: &StoredEvent) -> Result<()>;

    /// イベント保存（JSON文字列から）
    async fn save_event(&self, event_id: &str, event_json: &str) -> Result<()>;

    /// イベント取得
    async fn get_events(&self, filter: &StorageFilter) -> Result<Vec<StoredEvent>>;

    /// DMスレッド挿入/更新
    async fn upsert_dm_thread(&self, peer: &str, last_msg_at: i64) -> Result<()>;

    /// DMスレッド一覧取得
    async fn get_dm_threads(&self) -> Result<Vec<DmThread>>;

    /// 既読位置取得
    async fn get_last_seen(&self, scope: &str) -> Result<i64>;

    /// 既読位置設定
    async fn set_last_seen(&self, scope: &str, ts: i64) -> Result<()>;

    /// Outboxにキューイング
    async fn enqueue_outbox(&self, item: OutboxItem) -> Result<String>;

    /// Outboxステータス更新
    async fn update_outbox_status(&self, req_id: &str, status: OutboxStatus) -> Result<()>;

    /// 保留中のOutboxアイテム取得
    async fn get_pending_outbox(&self) -> Result<Vec<OutboxItem>>;

    /// 鍵ペア保存（内蔵Signer用）
    async fn save_keypair(&self, encrypted_data: &[u8]) -> Result<()>;

    /// 鍵ペア取得（内蔵Signer用）
    async fn get_keypair(&self) -> Result<Option<Vec<u8>>>;
}

