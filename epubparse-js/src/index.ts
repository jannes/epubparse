import { parse_epub } from "epubparse-wasm";
import { Book, Chapter, ChapterComponent } from "./data.js"

export { Book, Chapter, ChapterComponent }

export function epub_to_title(bytes: Uint8Array): string {
    return parse_epub(bytes)
}
  