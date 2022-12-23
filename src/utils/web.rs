use js_sys::{JsString, Math::random};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, Response, Window};

pub fn window() -> Window {
    web_sys::window().expect("window should exist")
}

pub fn document() -> Document {
    window().document().expect("document should exist")
}

pub async fn fetch_str(url: &str) -> Result<String, JsValue> {
    let res = JsFuture::from(window().fetch_with_str(url))
        .await?
        .dyn_into::<Response>()?;
    let text = JsFuture::from(res.text()?).await?.dyn_into::<JsString>()?;
    Ok(String::from(text))
}

pub fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) -> Result<i32, JsValue> {
    window().request_animation_frame(f.as_ref().unchecked_ref())
}

pub fn random_f32() -> f32 {
    random() as f32
}
