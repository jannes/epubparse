use serde::{Deserialize, Serialize};

/// A text-only book
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub author: Option<String>,
    pub preface_content: String,
    pub chapters: Vec<Chapter>,
}

/// A chapter within a book
///
/// A chapter has a title and content  
/// The content is sequentially made up of
///     1. text (may be empty)
///     2. a sequence of subchapters (may be zero)
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub text: String,
    pub subchapters: Vec<Chapter>,
}
