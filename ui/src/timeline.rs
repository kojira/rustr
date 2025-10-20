use eframe::egui;
use core::types::UiRow;
use crate::i18n::I18n;

/// タイムライン表示
pub struct Timeline {
    events: Vec<UiRow>,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
    
    /// イベントを追加
    pub fn add_event(&mut self, event: UiRow) {
        // 新しいイベントを先頭に追加（最新が上）
        self.events.insert(0, event);
        
        // 最大1000件まで保持
        if self.events.len() > 1000 {
            self.events.truncate(1000);
        }
    }
    
    /// イベント数を取得
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
    
    /// チャンネルを読み込み（イベントをクリア）
    pub fn load_channel(&mut self, _channel_id: &str) {
        self.events.clear();
    }
    
    /// DMを読み込み（イベントをクリア）
    pub fn load_dm(&mut self, _peer: &str) {
        self.events.clear();
    }
    
    /// タイムライン表示
    pub fn show(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.events.is_empty() {
                    ui.centered_and_justified(|ui| {
                        crate::emoji_label::emoji_label(ui, i18n.timeline_empty());
                    });
                    return;
                }
                
                for event in &self.events {
                    self.show_event(ui, event, i18n);
                    ui.separator();
                }
            });
    }
    
    /// 個別イベント表示
    fn show_event(&self, ui: &mut egui::Ui, event: &UiRow, i18n: &I18n) {
        ui.horizontal(|ui| {
            // アバター（仮）
            crate::emoji_label::emoji_label(ui, "👤");
            
            ui.vertical(|ui| {
                // ヘッダー（pubkey + 時刻）
                ui.horizontal(|ui| {
                    let pubkey_text = egui::RichText::new(&event.pubkey).strong();
                    egui_twemoji::EmojiLabel::new(pubkey_text).show(ui);
                    crate::emoji_label::emoji_label(ui, format_timestamp(event.created_at));
                });
                
                // コンテンツ（カラー絵文字対応）
                crate::emoji_label::emoji_label(ui, &event.content);
                
                // アクション
                ui.horizontal(|ui| {
                    if ui.button(i18n.timeline_reply()).clicked() {
                        log::info!("Reply to event");
                    }
                    if ui.button(i18n.timeline_like()).clicked() {
                        log::info!("Like event");
                    }
                });
            });
        });
    }
}

/// タイムスタンプをフォーマット
fn format_timestamp(timestamp: i64) -> String {
    let now = js_sys::Date::now() / 1000.0;
    let diff = (now as i64 - timestamp).abs() as u64;
    
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
