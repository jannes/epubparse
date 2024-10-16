use epubparse::epub_to_book;
use wasm_bindgen::prelude::*;

/// returns either
/// - Book converted to JsValue
/// - ParseError converted to JsValue
#[wasm_bindgen]
pub fn parse_epub(bytes: &[u8]) -> Result<JsValue, JsValue> {
    let book = epub_to_book(bytes);
    book.map(|b| serde_wasm_bindgen::to_value(&b).unwrap())
        .map_err(|parse_error| JsValue::from_str(&parse_error.to_string()))
}
