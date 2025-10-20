use eframe::egui;

/// メッセージ作成UI
pub struct Composer {
    text: String,
}

impl Composer {
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }
    
    /// コンポーザーを表示
    /// 送信ボタンが押されたら Some(content) を返す
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<String> {
        let mut should_send = false;
        
        ui.vertical(|ui| {
            crate::emoji_label::emoji_label(ui, "✏ Compose Message");
            
            // テキスト入力
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .hint_text("Type your message here...")
            );
            
            // Enter + Ctrl/Cmd で送信
            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command) {
                should_send = true;
            }
            
            ui.horizontal(|ui| {
                if ui.button("📤 Send").clicked() {
                    should_send = true;
                }
                
                crate::emoji_label::emoji_label(ui, format!("{} chars", self.text.len()));
            });
        });
        
        if should_send && !self.text.trim().is_empty() {
            let content = self.text.clone();
            self.text.clear();
            Some(content)
        } else {
            None
        }
    }
    
    pub fn clear(&mut self) {
        self.text.clear();
    }
}
