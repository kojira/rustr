use std::collections::HashMap;
use serde_json::{json, Value};

use crate::types::TimeWindow;

/// 購読の状態
#[derive(Debug, Clone)]
pub struct ActiveSub {
    pub sub_id: String,
    pub filter_json: String,
    pub eose_count: u32,
    pub last_extended_at: i64,
}

/// 購読マネージャー
pub struct SubscriptionManager {
    active_subs: HashMap<String, ActiveSub>,
    channel_windows: HashMap<String, TimeWindow>,
    dm_windows: HashMap<String, TimeWindow>,
    self_pubkey: Option<String>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            active_subs: HashMap::new(),
            channel_windows: HashMap::new(),
            dm_windows: HashMap::new(),
            self_pubkey: None,
        }
    }

    pub fn set_self_pubkey(&mut self, pubkey: String) {
        self.self_pubkey = Some(pubkey);
    }

    /// チャンネルを開く
    pub fn open_channel(&mut self, channel_id: &str) -> Vec<(String, String)> {
        let now = current_timestamp();
        let since = now - 600; // 初回は10分前から

        let window = TimeWindow::new(since);
        self.channel_windows.insert(channel_id.to_string(), window);

        // サブスクリプションIDを生成
        let sub_id = format!("ch:{}", channel_id);
        let filter = json!({
            "kinds": [42], // NIP-28 channel message
            "#e": [channel_id],
            "since": since,
        });

        let filter_json = filter.to_string();
        self.active_subs.insert(
            sub_id.clone(),
            ActiveSub {
                sub_id: sub_id.clone(),
                filter_json: filter_json.clone(),
                eose_count: 0,
                last_extended_at: now,
            },
        );

        vec![(sub_id, filter_json)]
    }

    /// DMスレッドを開く
    pub fn open_dm(&mut self, peer: &str, self_pubkey: &str) -> Vec<(String, String)> {
        let now = current_timestamp();
        let since = now - 600; // 初回は10分前から

        let window = TimeWindow::new(since);
        self.dm_windows.insert(peer.to_string(), window);

        // サブスクリプションIDを生成
        let sub_id = format!("dm:{}", peer);
        
        // NIP-04: kind=4, authors=[self] OR #p=[self]
        let filter = json!({
            "kinds": [4],
            "authors": [self_pubkey],
            "#p": [peer],
            "since": since,
        });

        let filter_json = filter.to_string();
        self.active_subs.insert(
            sub_id.clone(),
            ActiveSub {
                sub_id: sub_id.clone(),
                filter_json: filter_json.clone(),
                eose_count: 0,
                last_extended_at: now,
            },
        );

        // 逆方向のフィルター（peer -> self）
        let sub_id2 = format!("dm:{}:r", peer);
        let filter2 = json!({
            "kinds": [4],
            "authors": [peer],
            "#p": [self_pubkey],
            "since": since,
        });

        let filter_json2 = filter2.to_string();
        self.active_subs.insert(
            sub_id2.clone(),
            ActiveSub {
                sub_id: sub_id2.clone(),
                filter_json: filter_json2.clone(),
                eose_count: 0,
                last_extended_at: now,
            },
        );

        vec![(sub_id, filter_json), (sub_id2, filter_json2)]
    }

    /// EOSE受信時の処理
    pub fn on_eose(&mut self, sub_id: &str) -> Option<Vec<(String, String)>> {
        if let Some(sub) = self.active_subs.get_mut(sub_id) {
            sub.eose_count += 1;

            // 段階的に窓を拡大
            if self.should_extend_window(sub_id) {
                return self.extend_window(sub_id);
            }
        }
        None
    }

    /// EOSEをマーク（on_eoseのエイリアス）
    pub fn mark_eose(&mut self, sub_id: &str) {
        if let Some(sub) = self.active_subs.get_mut(sub_id) {
            sub.eose_count += 1;
        }
    }

    /// ウィンドウ拡張が必要か
    pub fn needs_extension(&self, sub_id: &str) -> bool {
        self.should_extend_window(sub_id)
    }

    /// 窓を拡大すべきか
    pub fn should_extend_window(&self, sub_id: &str) -> bool {
        if let Some(sub) = self.active_subs.get(sub_id) {
            // 最大4段階まで拡大
            sub.eose_count < 4
        } else {
            false
        }
    }

    /// 窓を拡大
    pub fn extend_window(&mut self, sub_id: &str) -> Option<Vec<(String, String)>> {
        let sub = self.active_subs.get(sub_id)?;
        let eose_count = sub.eose_count;

        // 拡大量: 1h -> 1d -> 7d -> 30d
        let additional_seconds = match eose_count {
            1 => 3600,        // 1時間
            2 => 86400,       // 1日
            3 => 604800,      // 7日
            4 => 2592000,     // 30日
            _ => return None,
        };

        // チャンネルまたはDMの窓を更新
        let new_since = if sub_id.starts_with("channel:") {
            let channel_id = sub_id.strip_prefix("channel:")?;
            if let Some(window) = self.channel_windows.get_mut(channel_id) {
                window.extend(additional_seconds);
                Some(window.since)
            } else {
                None
            }
        } else if sub_id.starts_with("dm:") {
            let parts: Vec<&str> = sub_id.splitn(2, ':').collect();
            if parts.len() >= 2 {
                let peer = parts[1].split(':').next()?;
                if let Some(window) = self.dm_windows.get_mut(peer) {
                    window.extend(additional_seconds);
                    Some(window.since)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(since) = new_since {
            return self.create_extended_filter(sub_id, since);
        }

        None
    }

    /// 拡大されたフィルターを作成
    fn create_extended_filter(&mut self, sub_id: &str, new_since: i64) -> Option<Vec<(String, String)>> {
        let sub = self.active_subs.get_mut(sub_id)?;
        
        // 既存のフィルターをパースして since を更新
        if let Ok(mut filter) = serde_json::from_str::<Value>(&sub.filter_json) {
            filter["since"] = json!(new_since);
            let new_filter_json = filter.to_string();
            
            sub.filter_json = new_filter_json.clone();
            sub.last_extended_at = current_timestamp();

            Some(vec![(sub_id.to_string(), new_filter_json)])
        } else {
            None
        }
    }

    /// アクティブな購読を取得
    pub fn get_active_subs(&self) -> Vec<&ActiveSub> {
        self.active_subs.values().collect()
    }

    /// 購読をクローズ
    pub fn close_subscription(&mut self, sub_id: &str) {
        self.active_subs.remove(sub_id);
    }
}

/// 現在のUNIXタイムスタンプ（秒）
fn current_timestamp() -> i64 {
    (js_sys::Date::now() / 1000.0) as i64
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_open_channel() {
        let mut mgr = SubscriptionManager::new();
        let filters = mgr.open_channel("test_channel");
        
        assert_eq!(filters.len(), 1);
        assert!(filters[0].0.starts_with("channel:"));
        
        let filter: Value = serde_json::from_str(&filters[0].1).unwrap();
        assert_eq!(filter["kinds"][0], 42);
        assert_eq!(filter["#e"][0], "test_channel");
    }

    #[wasm_bindgen_test]
    fn test_open_dm() {
        let mut mgr = SubscriptionManager::new();
        let self_pubkey = "self123";
        let peer = "peer456";
        
        let filters = mgr.open_dm(peer, self_pubkey);
        
        assert_eq!(filters.len(), 2);
        assert!(filters[0].0.starts_with("dm:"));
        
        let filter1: Value = serde_json::from_str(&filters[0].1).unwrap();
        assert_eq!(filter1["kinds"][0], 4);
        assert_eq!(filter1["authors"][0], self_pubkey);
        assert_eq!(filter1["#p"][0], peer);
    }

    #[wasm_bindgen_test]
    fn test_eose_extension() {
        let mut mgr = SubscriptionManager::new();
        mgr.open_channel("test");
        
        let sub_id = "channel:test";
        
        // 1回目のEOSE
        let result = mgr.on_eose(sub_id);
        assert!(result.is_some());
        
        let sub = mgr.active_subs.get(sub_id).unwrap();
        assert_eq!(sub.eose_count, 1);
    }
}

