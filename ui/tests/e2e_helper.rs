// E2Eテスト用のヘルパー
// eGuiはcanvasベースなので、通常のDOM操作ではテストできない
// 代わりに、アプリケーションの状態をJavaScript経由で確認する

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// アプリケーションの状態を取得するヘルパー
#[wasm_bindgen]
pub struct TestHelper;

#[wasm_bindgen]
impl TestHelper {
    /// アプリケーションが起動したかチェック
    pub fn is_app_running() -> bool {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector("canvas").ok())
            .flatten()
            .is_some()
    }
    
    /// Canvasのサイズを取得
    pub fn get_canvas_size() -> JsValue {
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector("canvas").ok())
            .flatten()
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok());
        
        if let Some(canvas) = canvas {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"width".into(), &canvas.width().into()).unwrap();
            js_sys::Reflect::set(&obj, &"height".into(), &canvas.height().into()).unwrap();
            obj.into()
        } else {
            JsValue::NULL
        }
    }
    
    /// Canvasをクリック（座標指定）
    pub fn click_canvas(x: i32, y: i32) -> bool {
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector("canvas").ok())
            .flatten()
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok());
        
        if let Some(canvas) = canvas {
            let event = web_sys::MouseEvent::new_with_mouse_event_init_dict(
                "click",
                web_sys::MouseEventInit::new()
                    .bubbles(true)
                    .client_x(x)
                    .client_y(y)
            ).unwrap();
            
            canvas.dispatch_event(&event).unwrap_or(false)
        } else {
            false
        }
    }
}

#[wasm_bindgen_test]
fn test_app_starts() {
    assert!(TestHelper::is_app_running(), "App should be running");
}

#[wasm_bindgen_test]
fn test_canvas_exists() {
    let size = TestHelper::get_canvas_size();
    assert!(!size.is_null(), "Canvas should exist");
}

