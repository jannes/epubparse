# Epubparse-rs

WORK IN PROGRESS

Requires Rust 1.56 to compile

This library aims to convert Epub files into text-only Book structures
that can be used to do analysis of the contained text.

## Design goals
- perform a reasonable conversion into a book with chapters ✅
- support Epub version 2 ✅
- support Epub version 3 ❌ 
  (most version 3 epubs also include version 2 .ncx table of contents, these should also work)
- serve as core to a epubparse-wasm library (must compile to WASM) ✅