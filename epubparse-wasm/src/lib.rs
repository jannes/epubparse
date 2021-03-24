mod utils;

extern crate web_sys;
extern crate epubparse;

use std::io::Cursor;

use epubparse::parse::EpubArchive;
use js_sys::Array;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Array<string>")]
    type StringArray;
}

// #[wasm_bindgen]
// pub struct BookExp {
//     pub title: String, 
//     pub author: String, 
//     pub chapter_titles: StringArray,
//     pub chapter_contents: StringArray,
// }


#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, epubparse!");
}

#[wasm_bindgen]
pub fn parse_epub(bytes: &[u8]) -> String {
    let epub_archive = EpubArchive::<Cursor<&[u8]>>::from_bytes(bytes);
    match epub_archive {
        Ok(ea) => ea.get_title().to_string(),
        Err(e) => {
            log!("err");
            e.to_string()
        }
    }
}
