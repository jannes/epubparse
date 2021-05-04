export interface Book {
    title: string
    author: string | undefined
    prefaceContent: string
    chapters: Array<Chapter>
}

export interface Chapter {
    title: string
    text: string
    subchapters: Array<Chapter>
}