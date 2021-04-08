/// A text-only book
pub struct Book {
    pub title: String,
    pub author: String,
    pub chapters: Vec<Chapter>,
}

/// A chapter within a book
///
/// A chapter has a title and content  
/// The content is made up of sections
pub struct Chapter {
    pub title: String,
    pub content: Vec<Section>,
}

/// A section within a chapter
///
/// A section in a chapter is either
/// - a chunk of text that belongs to the current chapter
/// - a subchapter 
pub enum Section {
    Chunk(String),
    SubChapter(Chapter),
}