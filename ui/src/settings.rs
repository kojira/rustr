use crate::font_config::{FontConfig, FontFamily};

/// 設定画面
pub struct SettingsView {
    font_config: FontConfig,
    font_changed: bool,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            font_config: FontConfig::load(),
            font_changed: false,
        }
    }

    /// 設定画面を表示
    pub fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        crate::emoji_label::emoji_heading(ui, "⚙️ 設定");
        ui.add_space(20.0);

        ui.group(|ui| {
            crate::emoji_label::emoji_label(ui, "🔤 フォント設定");
            ui.add_space(10.0);

            let current_font = self.font_config.font_family;

            egui::ComboBox::from_label("フォントファミリー")
                .selected_text(current_font.name())
                .show_ui(ui, |ui| {
                    for font in FontFamily::all() {
                        let response = ui.selectable_value(
                            &mut self.font_config.font_family,
                            *font,
                            font.name(),
                        );
                        if response.clicked() {
                            self.font_changed = true;
                        }
                    }
                });

            if self.font_changed {
                ui.add_space(10.0);
                let warning_text = egui::RichText::new("⚠️ フォント変更を適用するには再起動が必要です").color(egui::Color32::YELLOW);
                crate::emoji_label::emoji_label(ui, warning_text);

                if ui.button("💾 保存して再起動").clicked() {
                    self.font_config.save();
                    // ページをリロード
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().reload();
                    }
                }
            }
        });

        ui.add_space(20.0);

        ui.group(|ui| {
            crate::emoji_label::emoji_label(ui, "ℹ️ フォント情報");
            ui.add_space(10.0);

            match self.font_config.font_family {
                FontFamily::NotoSansJP => {
                    crate::emoji_label::emoji_label(ui, "Noto Sans JP");
                    crate::emoji_label::emoji_label(ui, "• 日本語完全対応");
                    crate::emoji_label::emoji_label(ui, "• 絵文字対応");
                    crate::emoji_label::emoji_label(ui, "• 読みやすいゴシック体");
                }
                FontFamily::SystemDefault => {
                    crate::emoji_label::emoji_label(ui, "System Default");
                    crate::emoji_label::emoji_label(ui, "• システムのデフォルトフォント");
                    crate::emoji_label::emoji_label(ui, "• 日本語は表示されない場合があります");
                }
            }
        });

        ui.add_space(20.0);

        // フォントプレビュー
        ui.group(|ui| {
            crate::emoji_label::emoji_label(ui, "📝 プレビュー");
            ui.add_space(10.0);

            crate::emoji_label::emoji_label(ui, "日本語: こんにちは、世界！");
            crate::emoji_label::emoji_label(ui, "English: Hello, World!");
            crate::emoji_label::emoji_label(ui, "絵文字: 🎉 🚀 ✨ 💡 🔥");
            crate::emoji_label::emoji_label(ui, "数字: 0123456789");
        });
    }

    /// フォント設定を取得
    pub fn font_config(&self) -> &FontConfig {
        &self.font_config
    }
}
