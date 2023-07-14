use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// Requests user attention.
pub async fn alert_window() {
    log::info!("timer finished: requesting user attention");
    invoke("alert_window", JsValue::UNDEFINED).await;
}
