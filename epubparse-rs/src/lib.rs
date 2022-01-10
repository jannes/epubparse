//! A library to parse epub files
//!
//! Design goals:
//! - ✅ serve as core to the epubparse-wasm library (must compile to WASM)
//! - ✅ perform a reasonable conversion into a book with chapters
//! - ✅ support Epub version 2 table of contents (.ncx)
//! - ❌ support Epub version 3 table of contents (.xhtml) (not yet implemented, but
//!   many version 3 epubs also include version 2 table of contents, these should also work)

use errors::ParseError;
use parse::EpubArchive;
use types::Book;

pub mod errors;
mod html_entities;
mod parse;
pub mod types;
mod util;

/// Parse an epub file to a text-only book structure
///
/// This may fail due to various reasons, that are captured by the returned Result's Error type
pub fn epub_to_book(bytes: &[u8]) -> Result<Book, ParseError> {
    EpubArchive::new(bytes).and_then(|archive| archive.to_book())
}

#[cfg(test)]
mod tests {
    use super::*;

    static EPUB_PAID_OFF: &[u8] = include_bytes!("../../test_resources/paid_off.epub");

    #[test]
    fn epub_to_book_paid_off() {
        let book = epub_to_book(EPUB_PAID_OFF).unwrap();
        assert_eq!("Paid Off", &book.title);
    }
}
