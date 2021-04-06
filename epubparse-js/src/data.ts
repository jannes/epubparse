export interface Book {
    title: string
    chapters: Array<Chapter>
    nestingDepth: number
}

export interface Chapter {
    title: string
    components: Array<ChapterComponent>
}

export type ChapterComponent = Chapter | string