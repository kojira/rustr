pub mod types;
pub mod storage;
pub mod relay;
pub mod subscription;
pub mod outbox;
pub mod signer;
pub mod error;

use std::collections::VecDeque;
use std::sync::Arc;

pub use error::{CoreError, Result};

use crate::storage::Storage;
use crate::relay::{RelayConnection, RelayMessage};
use crate::subscription::SubscriptionManager;
use crate::outbox::OutboxQueue;
use crate::signer::Signer;
use crate::types::UiRow;

/// CoreHandle: UIから使用されるメインAPI
pub struct CoreHandle {
    relays: Vec<RelayConnection>,
    sub_mgr: SubscriptionManager,
    outbox: OutboxQueue,
    storage: Arc<dyn Storage>,
    signer: Option<Arc<dyn Signer>>,
    event_buffer: VecDeque<UiRow>,
}

impl CoreHandle {
    /// 初期化
    pub async fn init(relay_urls: Vec<String>, storage: Arc<dyn Storage>) -> Result<Self> {
        let relays = relay_urls
            .into_iter()
            .map(|url| RelayConnection::new(url))
            .collect();

        let sub_mgr = SubscriptionManager::new();
        let outbox = OutboxQueue::new(storage.clone());

        Ok(Self {
            relays,
            sub_mgr,
            outbox,
            storage,
            signer: None,
            event_buffer: VecDeque::new(),
        })
    }

    /// Signerを設定
    pub fn set_signer(&mut self, signer: Arc<dyn Signer>) {
        self.signer = Some(signer);
    }

    /// 公開鍵を取得
    pub async fn get_public_key(&self) -> Result<Option<String>> {
        if let Some(signer) = &self.signer {
            Ok(Some(signer.get_public_key().await?))
        } else {
            Ok(None)
        }
    }

    /// 全Relayに接続
    pub async fn connect_all(&mut self) -> Result<()> {
        for relay in &mut self.relays {
            if let Err(e) = relay.connect().await {
                log::error!("Failed to connect to {}: {:?}", relay.url, e);
            }
        }
        Ok(())
    }

    /// チャンネルを開く
    pub async fn open_channel(&mut self, channel_id: &str) -> Result<()> {
        let filters = self.sub_mgr.open_channel(channel_id);
        
        // 全Relayに購読リクエスト送信
        for (sub_id, filter_json) in filters {
            let req = format!(r#"["REQ","{}",{}]"#, sub_id, filter_json);
            for relay in &self.relays {
                let _ = relay.send(&req).await;
            }
        }
        Ok(())
    }

    /// DMスレッドを開く
    pub async fn open_dm(&mut self, peer: &str) -> Result<()> {
        let self_pubkey = if let Some(pk) = self.get_public_key().await? {
            pk
        } else {
            return Err(CoreError::Other("No signer available".to_string()));
        };
        
        let filters = self.sub_mgr.open_dm(peer, &self_pubkey);
        
        // 全Relayに購読リクエスト送信
        for (sub_id, filter_json) in filters {
            let req = format!(r#"["REQ","{}",{}]"#, sub_id, filter_json);
            for relay in &self.relays {
                let _ = relay.send(&req).await;
            }
        }
        Ok(())
    }

    /// チャンネル作成 (NIP-28)
    pub async fn create_channel(&mut self, name: &str, about: &str, picture: &str) -> Result<String> {
        let signer = self.signer.as_ref()
            .ok_or_else(|| CoreError::Other("No signer available".to_string()))?;
        
        // NIP-28: チャンネル作成 (kind 40)
        let content = serde_json::json!({
            "name": name,
            "about": about,
            "picture": picture,
        }).to_string();
        
        let unsigned_event = crate::signer::UnsignedEvent {
            kind: 40,
            content,
            tags: vec![],
            created_at: (js_sys::Date::now() / 1000.0) as i64,
        };
        
        let signed_event = signer.sign_event(unsigned_event).await?;
        let event_id = signed_event.id.clone();
        let event_json = signed_event.to_json();
        
        // Outboxキューに追加
        self.outbox.enqueue(event_json).await?;
        
        Ok(event_id)
    }

    /// パブリックメッセージ送信
    pub async fn send_public(&mut self, channel_id: &str, content: &str) -> Result<String> {
        let signer = self.signer.as_ref()
            .ok_or_else(|| CoreError::Other("No signer available".to_string()))?;
        
        // NIP-28: チャンネルメッセージ (kind 42)
        // eタグ: ["e", <32-bytes lowercase hex of the id of another event>, <recommended relay URL, optional>]
        let tags = vec![
            vec!["e".to_string(), channel_id.to_string()],
        ];
        
        let unsigned_event = crate::signer::UnsignedEvent {
            kind: 42,
            content: content.to_string(),
            tags,
            created_at: (js_sys::Date::now() / 1000.0) as i64,
        };
        
        let signed_event = signer.sign_event(unsigned_event).await?;
        let event_id = signed_event.id.clone();
        let event_json = signed_event.to_json();
        
        // Outboxキューに追加
        self.outbox.enqueue(event_json).await?;
        
        Ok(event_id)
    }

    /// DM送信
    pub async fn send_dm(&mut self, peer: &str, plaintext: &str) -> Result<String> {
        let signer = self.signer.as_ref()
            .ok_or_else(|| CoreError::Other("No signer available".to_string()))?;
        
        // NIP-04暗号化
        let encrypted = signer.nip04_encrypt(peer, plaintext).await?;
        
        // NIP-04: DM (kind 4)
        let tags = vec![
            vec!["p".to_string(), peer.to_string()],
        ];
        
        let unsigned_event = crate::signer::UnsignedEvent {
            kind: 4,
            content: encrypted,
            tags,
            created_at: (js_sys::Date::now() / 1000.0) as i64,
        };
        
        let signed_event = signer.sign_event(unsigned_event).await?;
        let event_id = signed_event.id.clone();
        let event_json = signed_event.to_json();
        
        // Outboxキューに追加
        self.outbox.enqueue(event_json).await?;
        
        Ok(event_id)
    }

    /// UIイベントをポーリング
    pub fn poll_events(&mut self, max: u32) -> Vec<UiRow> {
        let mut result = Vec::new();
        for _ in 0..max {
            if let Some(row) = self.event_buffer.pop_front() {
                result.push(row);
            } else {
                break;
            }
        }
        result
    }

    /// 定期処理（再接続、送信キュー処理等）
    pub async fn tick(&mut self) -> Result<()> {
        // Relay再接続チェック
        for relay in &mut self.relays {
            if relay.needs_reconnect() {
                let _ = relay.connect().await;
            }
        }

        // 受信メッセージ処理
        let mut all_messages = Vec::new();
        for relay in &mut self.relays {
            all_messages.extend(relay.drain_messages());
        }
        for msg in all_messages {
            self.process_relay_message(msg).await
                .map_err(|e| {
                    log::error!("tick: Error processing relay message: {:?}", e);
                    e
                })?;
        }

        // Outbox処理（送信キューからイベントを取り出して送信）
        match self.outbox.dequeue().await {
            Ok(Some(event_json)) => {
                let msg = format!(r#"["EVENT",{}]"#, event_json);
                log::info!("Sending EVENT to relays: {}", msg);
                
                for relay in &self.relays {
                    if let Err(e) = relay.send(&msg).await {
                        log::error!("Failed to send to relay {}: {:?}", relay.url, e);
                    } else {
                        log::info!("Sent to relay: {}", relay.url);
                    }
                }
            }
            Ok(None) => {
                // キューが空の場合は何もしない
            }
            Err(e) => {
                log::error!("tick: Error in outbox.dequeue(): {:?}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Relayメッセージを処理
    async fn process_relay_message(&mut self, msg: RelayMessage) -> Result<()> {
        match msg {
            RelayMessage::Event { sub_id: _, event_json } => {
                // イベントをパース
                let event: serde_json::Value = serde_json::from_str(&event_json)
                    .map_err(|e| CoreError::ParseError(e.to_string()))?;
                
                // ストレージに保存
                let event_id = event["id"].as_str().unwrap_or("");
                self.storage.save_event(event_id, &event_json).await?;
                
                // UIバッファに追加
                let kind = event["kind"].as_u64().unwrap_or(0) as u16;
                let content = event["content"].as_str().unwrap_or("").to_string();
                let created_at = event["created_at"].as_i64().unwrap_or(0);
                let pubkey = event["pubkey"].as_str().unwrap_or("").to_string();
                
                let ui_row = UiRow {
                    id: event_id.to_string(),
                    kind,
                    pubkey,
                    created_at,
                    content,
                    image_url: None,
                };
                
                self.event_buffer.push_back(ui_row);
            }
            RelayMessage::Eose { sub_id } => {
                self.sub_mgr.mark_eose(&sub_id);
                
                // ウィンドウ拡張が必要か確認
                if self.sub_mgr.needs_extension(&sub_id) {
                    if let Some(filters) = self.sub_mgr.extend_window(&sub_id) {
                        for (new_sub_id, filter_json) in filters {
                            let req = format!(r#"["REQ","{}",{}]"#, new_sub_id, filter_json);
                            for relay in &self.relays {
                                let _ = relay.send(&req).await;
                            }
                        }
                    }
                }
            }
            RelayMessage::Ok { event_id, accepted, message } => {
                if accepted {
                    log::info!("Event {} accepted", event_id);
                    // Outboxから削除
                    self.outbox.on_ok(&event_id, true, "").await?;
                } else {
                    log::warn!("Event {} rejected: {}", event_id, message);
                    // エラーステータスに変更
                    self.outbox.on_ok(&event_id, false, &message).await?;
                }
            }
            RelayMessage::Notice { message } => {
                log::info!("Relay notice: {}", message);
            }
        }
        Ok(())
    }
}

