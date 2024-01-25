use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
extern crate console_error_panic_hook;
mod deserializer;

use console_error_panic_hook::set_once as set_panic_hook;

#[wasm_bindgen]
pub fn parse(document: &str) -> JsValue {
    set_panic_hook();

    let result = deserializer::parser(document);

    serde_wasm_bindgen::to_value(&result).unwrap()
}
