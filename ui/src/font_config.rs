use serde::{Deserialize, Serialize};

/// フォント設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub font_family: FontFamily,
}

/// 利用可能なフォントファミリー
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontFamily {
    /// Noto Sans JP（日本語・絵文字対応）
    NotoSansJP,
    /// システムデフォルト
    SystemDefault,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            font_family: FontFamily::NotoSansJP,
        }
    }
}

impl FontFamily {
    pub fn name(&self) -> &'static str {
        match self {
            FontFamily::NotoSansJP => "Noto Sans JP",
            FontFamily::SystemDefault => "System Default",
        }
    }

    pub fn all() -> &'static [FontFamily] {
        &[FontFamily::NotoSansJP, FontFamily::SystemDefault]
    }
}

impl FontConfig {
    const STORAGE_KEY: &'static str = "font_config";

    /// LocalStorageから読み込み
    pub fn load() -> Self {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item(Self::STORAGE_KEY) {
                    if let Ok(config) = serde_json::from_str(&json) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    /// LocalStorageに保存
    pub fn save(&self) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(json) = serde_json::to_string(self) {
                    let _ = storage.set_item(Self::STORAGE_KEY, &json);
                }
            }
        }
    }

    /// フォントをeGuiに適用
    pub fn apply_to_egui(&self, ctx: &egui::Context) {
        match self.font_family {
            FontFamily::NotoSansJP => {
                // デフォルトのフォント定義を取得
                let mut fonts = egui::FontDefinitions::default();
                
                // Noto Sans JPフォントデータを埋め込み（日本語用）
                let noto_sans_jp = include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf");
                fonts.font_data.insert(
                    "NotoSansJP".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_static(noto_sans_jp)),
                );

                // Noto Emojiフォントデータを埋め込み（絵文字用）
                // notedeckと同じ設定を使用
                let noto_emoji = include_bytes!("../assets/fonts/NotoEmoji-Regular.ttf");
                fonts.font_data.insert(
                    "NotoEmoji".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_static(noto_emoji)
                        .tweak(egui::FontTweak {
                            scale: 1.1, // 少し大きく
                            y_offset_factor: 0.0,
                            y_offset: 0.0,
                            baseline_offset_factor: 0.0,
                        })),
                );

                // Proportionalファミリーの優先順位を設定
                // 1. NotoSansJP (日本語)
                // 2. NotoEmoji (絵文字)
                // 3. その他のデフォルトフォント
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "NotoSansJP".to_owned());
                
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(1, "NotoEmoji".to_owned());

                // Monospaceファミリーにも追加（コードブロック用）
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Monospace)
                    .unwrap()
                    .insert(0, "NotoSansJP".to_owned());
                
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Monospace)
                    .unwrap()
                    .insert(1, "NotoEmoji".to_owned());
                
                ctx.set_fonts(fonts);
            }
            FontFamily::SystemDefault => {
                // デフォルトフォントを使用（何もしない）
            }
        }
    }
}

