use crate::font_config::{FontConfig, FontFamily};
use crate::i18n::{I18n, Language};

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
    pub fn show(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, i18n: &mut I18n) {
        crate::emoji_label::emoji_heading(ui, i18n.settings_title());
        ui.add_space(20.0);

        // 言語設定
        ui.group(|ui| {
            crate::emoji_label::emoji_label(ui, i18n.settings_language());
            ui.add_space(10.0);

            let current_lang = i18n.language();
            let mut selected_lang = *i18n.language();
            egui::ComboBox::from_label("")
                .selected_text(selected_lang.name())
                .show_ui(ui, |ui| {
                    for lang in Language::all() {
                        if ui.selectable_value(&mut selected_lang, *lang, lang.name()).clicked() {
                            i18n.set_language(*lang);
                        }
                    }
                });
        });

        ui.add_space(20.0);

        // フォント設定
        ui.group(|ui| {
            crate::emoji_label::emoji_label(ui, i18n.settings_font());
            ui.add_space(10.0);

            let current_font = self.font_config.font_family;

            egui::ComboBox::from_label(i18n.settings_font_family())
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
                let warning_text = egui::RichText::new(i18n.settings_restart_required()).color(egui::Color32::YELLOW);
                crate::emoji_label::emoji_label(ui, warning_text);

                if ui.button(i18n.settings_save_and_restart()).clicked() {
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
            crate::emoji_label::emoji_label(ui, i18n.settings_font_info());
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
            crate::emoji_label::emoji_label(ui, i18n.settings_font_preview());
            ui.add_space(10.0);

            crate::emoji_label::emoji_label(ui, i18n.settings_preview_japanese());
            crate::emoji_label::emoji_label(ui, i18n.settings_preview_english());
            crate::emoji_label::emoji_label(ui, i18n.settings_preview_emoji());
            crate::emoji_label::emoji_label(ui, i18n.settings_preview_numbers());
        });
    }

    /// フォント設定を取得
    pub fn font_config(&self) -> &FontConfig {
        &self.font_config
    }
}
