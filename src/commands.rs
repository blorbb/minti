use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    fn listen(event: &str, handler: &Function) -> JsValue;
}

/// Requests user attention.
pub async fn alert_window() {
    log::info!("timer finished: requesting user attention");
    invoke("alert_window", JsValue::UNDEFINED).await;
}

pub async fn popup_contextmenu() {
    log::info!("opened contextmenu");
    invoke("contextmenu", JsValue::UNDEFINED).await;
}

pub async fn set_contextmenu_checkitem(path: &str, checked: bool) {
    log::info!("setting {path} to {checked}");
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"path".into(), &path.into()).unwrap();
    js_sys::Reflect::set(&obj, &"checked".into(), &checked.into()).unwrap();
    invoke("set_contextmenu_checkitem", obj.into()).await;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    event: String,
    pub payload: String,
}

impl Event {
    pub fn id(&self) -> &str {
        &self.event
    }
}

pub fn listen_event(event: &str, callback: impl Fn(Event) + 'static) {
    let handler = Closure::<dyn Fn(JsValue) + 'static>::new(Box::new(move |payload: JsValue| {
        log::info!("running in closure");
        let payload: Event = serde_wasm_bindgen::from_value(payload).unwrap();
        callback(payload);
    }) as Box<dyn Fn(JsValue)>);
    listen(event, handler.as_ref().unchecked_ref());
    handler.forget();
}
