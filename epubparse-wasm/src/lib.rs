mod utils;

extern crate epubparse;

use epubparse::epub_to_book;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// returns either Book converted to JsValue
/// or ParseError converted to JsValue
#[wasm_bindgen]
pub fn parse_epub(bytes: &[u8]) -> Result<JsValue, JsValue> {
    set_panic_hook();
    let book = epub_to_book(bytes);
    book.map(|b| JsValue::from_serde(&b).unwrap())
        .map_err(|parse_error| JsValue::from_str(&parse_error.to_string()))
}
