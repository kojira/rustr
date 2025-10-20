use eframe::egui;

/// タイムライン表示
pub struct Timeline {
    events: Vec<TimelineEvent>,
    scroll_offset: f32,
}

#[derive(Clone)]
struct TimelineEvent {
    id: String,
    pubkey: String,
    content: String,
    created_at: i64,
    kind: u16,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            scroll_offset: 0.0,
        }
    }
    
    /// チャンネルのイベントを読み込み
    pub fn load_channel(&mut self, channel_id: &str) {
        log::info!("Loading channel: {}", channel_id);
        // TODO: CoreHandle経由でイベントを取得
        
        // デモデータ
        self.events = vec![
            TimelineEvent {
                id: "1".to_string(),
                pubkey: "alice".to_string(),
                content: "Hello, Nostr!".to_string(),
                created_at: 1700000000,
                kind: 42,
            },
            TimelineEvent {
                id: "2".to_string(),
                pubkey: "bob".to_string(),
                content: "Welcome to the decentralized future!".to_string(),
                created_at: 1700000100,
                kind: 42,
            },
        ];
    }
    
    /// DMのイベントを読み込み
    pub fn load_dm(&mut self, peer: &str) {
        log::info!("Loading DM with: {}", peer);
        // TODO: CoreHandle経由でDMを取得
        
        self.events = vec![
            TimelineEvent {
                id: "dm1".to_string(),
                pubkey: peer.to_string(),
                content: "Hey, how are you?".to_string(),
                created_at: 1700000200,
                kind: 4,
            },
        ];
    }
    
    /// タイムライン表示
    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.events.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("No events yet. Start a conversation!");
                    });
                    return;
                }
                
                for event in &self.events {
                    self.show_event(ui, event);
                    ui.separator();
                }
            });
    }
    
    fn show_event(&self, ui: &mut egui::Ui, event: &TimelineEvent) {
        ui.horizontal(|ui| {
            // アバター（仮）
            ui.label("👤");
            
            ui.vertical(|ui| {
                // ヘッダー（pubkey + 時刻）
                ui.horizontal(|ui| {
                    ui.strong(&event.pubkey);
                    ui.label(format_timestamp(event.created_at));
                });
                
                // コンテンツ
                ui.label(&event.content);
                
                // アクション
                ui.horizontal(|ui| {
                    if ui.small_button("↩ Reply").clicked() {
                        log::info!("Reply to {}", event.id);
                    }
                    if ui.small_button("♥ Like").clicked() {
                        log::info!("Like {}", event.id);
                    }
                });
            });
        });
    }
}

fn format_timestamp(ts: i64) -> String {
    // TODO: 適切な時刻フォーマット
    let now = js_sys::Date::now() / 1000.0;
    let diff = now as i64 - ts;
    
    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}
