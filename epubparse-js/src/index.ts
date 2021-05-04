import { parse_epub } from "epubparse-wasm";
import { Book, Chapter } from "./data.js"

export { Book, Chapter }

// return either Book or error message
export function epubToBook(bytes: Uint8Array): Book | string {
    // parse_epub returns book object on success and throws string error on failure
    try {
        let book = parse_epub(bytes);
        return convertToBook(book)
    }
    catch (e) {
        return e
    }
}

// has to be called with valid book object
// which the parse_epub function is guaranteed to return
function convertToBook(book_object: any): Book {
    let author = book_object.author
    let chapters = book_object.chapters
    return {
        title: book_object.title,
        author: author,
        prefaceContent: book_object.preface_content,
        chapters: book_object.chapters.map((c: any) => convertToChapter(c)),
    }
}

function convertToChapter(chapter_object: any): Chapter {
    return {
        title: chapter_object.title,
        text: chapter_object.text,
        subchapters: chapter_object.subchapters.map((sc: any) => convertToChapter(sc)),
    }
}
