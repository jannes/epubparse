//! A library to parse epub files
//!
//! Design goals:
//! - parse an epub file into a simple text-only book structure 
//!   (implemented)
//! - parse an epub file into a low-level epub structure,
//!   that exposes all resources in an comprehensive API
//!   (later versions)
//!
//! Atm only epub version 2 is supported, but I'm planning to also support 3.0.1/3.2.
//! Most of the time newer versions include all the files needed to be 
//! backwards compatible with version 2 though, so the current implementation often works for version 3 too.

use errors::ParseError;
use parse::EpubArchive;
use types::Book;

pub mod errors;
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
