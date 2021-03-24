pub struct Book {
    pub title: String,
    pub author: String,
    pub chapters: Vec<Chapter>,
}

pub struct Chapter {
    pub title: String,
    pub content: String,
}