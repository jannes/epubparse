# Epubparse-rs

This library aims to convert Epub files into text-only Book structures
that can be used to do analysis of the contained text.

## Design goals
- support all Epub versions 2.0-3.2 
- perform a reasonable conversion into a book with chapters
- serve as core to a epubparse-wasm library (must compile to WASM)