//! A library to parse epub files
//!
//! WIP! not usable yet
//!
//! Design goals:
//! - parse an epub file into a simple text-only book structure
//!   (current focus)
//! - parse an epub file into a low-level epub structure,
//!   that exposes all resources in an comprehensive API
//!   (later versions)

use std::io::Cursor;

use errors::ParseError;
use parse::EpubArchive;
use types::Book;
pub mod errors;
mod parse;
pub mod types;

/// Parse an epub file to a text-only book structure (UNIMPLEMENTED!)
///
/// This may fail due to various reasons, that are captured by the 
/// returned Result's Error type
pub fn epub_to_book(bytes: &[u8]) -> Result<Book, ParseError> {
    EpubArchive::<Cursor<&[u8]>>::from_bytes(bytes).map(|archive| archive.to_book())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn epub_to_contentopf() {
        let bytes = fs::read("test_resources/paid_off.epub").unwrap();
        let book = epub_to_book(&bytes).unwrap();
        assert_eq!("Paid Off", &book.title);
    }
}
