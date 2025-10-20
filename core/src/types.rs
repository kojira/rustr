use serde::{Deserialize, Serialize};

/// UI表示用の行データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiRow {
    pub id: String,
    pub kind: u16,
    pub pubkey: String,
    pub created_at: i64,
    pub content: String,
    pub image_url: Option<String>,
}

/// 送信キューのアイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxItem {
    pub req_id: String,
    pub event_json: String,
    pub status: OutboxStatus,
    pub last_try_at: i64,
    pub retry_count: u32,
    pub error: Option<String>,
}

/// 送信ステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutboxStatus {
    Queued,
    Sent,
    Ok,
    Error,
}

/// Storage用のイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: String,
    pub kind: u16,
    pub pubkey: String,
    pub created_at: i64,
    pub content: String,
    pub tags: Vec<Vec<String>>,
    pub sig: String,
    pub relay_hint: Option<String>,
    pub inserted_at: i64,
}

/// Storage検索フィルター
#[derive(Debug, Clone, Default)]
pub struct StorageFilter {
    pub kinds: Option<Vec<u16>>,
    pub authors: Option<Vec<String>>,
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub limit: Option<u32>,
}

/// DMスレッド情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmThread {
    pub peer: String,
    pub last_seen: i64,
    pub last_msg_at: i64,
}

/// 時間窓
#[derive(Debug, Clone, Copy)]
pub struct TimeWindow {
    pub since: i64,
    pub until: Option<i64>,
}

impl TimeWindow {
    pub fn new(since: i64) -> Self {
        Self { since, until: None }
    }

    pub fn extend(&mut self, additional_seconds: i64) {
        self.since -= additional_seconds;
    }
}

