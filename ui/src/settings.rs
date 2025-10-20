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
        ui.heading("⚙️ 設定");
        ui.add_space(20.0);

        ui.group(|ui| {
            ui.label("🔤 フォント設定");
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
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠️ フォント変更を適用するには再起動が必要です",
                );

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
            ui.label("ℹ️ フォント情報");
            ui.add_space(10.0);

            match self.font_config.font_family {
                FontFamily::NotoSansJP => {
                    ui.label("Noto Sans JP");
                    ui.label("• 日本語完全対応");
                    ui.label("• 絵文字対応");
                    ui.label("• 読みやすいゴシック体");
                }
                FontFamily::SystemDefault => {
                    ui.label("System Default");
                    ui.label("• システムのデフォルトフォント");
                    ui.label("• 日本語は表示されない場合があります");
                }
            }
        });

        ui.add_space(20.0);

        // フォントプレビュー
        ui.group(|ui| {
            ui.label("📝 プレビュー");
            ui.add_space(10.0);

            ui.label("日本語: こんにちは、世界！");
            ui.label("English: Hello, World!");
            ui.label("絵文字: 🎉 🚀 ✨ 💡 🔥");
            ui.label("数字: 0123456789");
        });
    }

    /// フォント設定を取得
    pub fn font_config(&self) -> &FontConfig {
        &self.font_config
    }
}

