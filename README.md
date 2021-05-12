# Epubparse

WORK IN PROGRESS

This library aims to convert Epub files into text-only Book structures
that can be used to do analysis of the contained text.
It is published both as a Rust crate to crates.io and as a NPM package (ESM module) to npm.

## Design goals
- perform a reasonable conversion into a book with chapters ✅
- support Epub version 2 ✅
- support Epub version 3 ❌ 
  (most version 3 epubs also include version 2 .ncx table of contents, these should also work)

## Structure
- epubparse-rs: core Rust library that compiles to WASM
  (published to crates.io)
- epubparse-wasm: wrapper around Rust core that provides
  JS compatible data types from WASM functions
  (published to npm, only meant to be consumed by JS lib)
- epubparse-js: JS library with ergonomic API including
  Typescript definitions
  (published to npm, for use in Browser and Node.js)
