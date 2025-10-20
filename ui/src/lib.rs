mod app;
mod timeline;
mod composer;
mod onboarding;

#[cfg(feature = "debug-test")]
mod debug_test;

pub use app::NostrApp;

use wasm_bindgen::prelude::*;

/// WASM初期化とパニックフック設定
#[wasm_bindgen(start)]
pub fn start() {
    // パニック時にコンソールにスタックトレースを表示
    console_error_panic_hook::set_once();
    
    // ログ設定
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logger");
    
    log::info!("Rustr WASM initialized");
}

/// Webアプリケーションのエントリーポイント
#[wasm_bindgen]
pub async fn start_app(canvas_id: String) -> Result<(), JsValue> {
    use wasm_bindgen::JsCast;
    
    let document = web_sys::window()
        .ok_or("No window")?
        .document()
        .ok_or("No document")?;
    
    let canvas = document
        .get_element_by_id(&canvas_id)
        .ok_or("Canvas not found")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    let web_options = eframe::WebOptions::default();
    
    eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|cc| {
                // フォントサイズを大きくする
                let mut style = (*cc.egui_ctx.style()).clone();
                style.text_styles = [
                    (egui::TextStyle::Heading, egui::FontId::new(32.0, egui::FontFamily::Proportional)),
                    (egui::TextStyle::Body, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
                    (egui::TextStyle::Button, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
                    (egui::TextStyle::Small, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
                    (egui::TextStyle::Monospace, egui::FontId::new(18.0, egui::FontFamily::Monospace)),
                ].into();
                
                // スペーシングも調整
                style.spacing.item_spacing = egui::vec2(12.0, 12.0);
                style.spacing.button_padding = egui::vec2(16.0, 8.0);
                
                cc.egui_ctx.set_style(style);
                
                Ok(Box::new(NostrApp::new(cc)))
            }),
        )
        .await?;
    
    Ok(())
}
