# Epubparse-rs

⚠️  Work in progress  

Requires Rust 1.56 to compile

This library aims to convert Epub files into text-only Book structures
that can be used to do analysis of the contained text.
It is published both as a Rust crate to [crates.io](https://crates.io/crates/epubparse) 
and as a NPM package (ESM module) to [npm](https://www.npmjs.com/package/epubparse-js).
See the [project repo](https://github.com/jannes/epubparse) for all components.

## Design goals
- ✅ serve as core to the epubparse-wasm library (must compile to WASM)
- ✅ perform a reasonable conversion into a book with chapters
- ✅ support Epub version 2 table of contents (.ncx)
- ❌ support Epub version 3 table of contents (.xhtml) (not yet implemented, but  
  many version 3 epubs also include version 2 table of contents, these should also work)
