# Epubparse

⚠️  Work in progress  

[![NPM](https://img.shields.io/npm/v/epubparse-js)](https://www.npmjs.com/package/epubparse-js)
[![Crates.io](https://img.shields.io/crates/v/epubparse)](https://crates.io/crates/epubparse)
[![Docs.rs](https://img.shields.io/docsrs/epubparse)](https://docs.rs/epubparse/latest/epubparse/)


This library aims to convert Epub files into text-only Book structures
that can be used to do analysis of the contained text.
It is published both as a Rust crate to crates.io and as a NPM package (ESM module) to npm.

## Design goals
- ✅ perform a reasonable conversion into a book with chapters
- ✅ support Epub version 2 table of contents (.ncx)
- ❌ support Epub version 3 table of contents (.xhtml) (not yet implemented, but  
  many version 3 epubs also include version 2 table of contents, these should also work)

## Structure
- epubparse-rs: core Rust library that compiles to WASM
  (published to crates.io)
- epubparse-wasm: wrapper around Rust core that provides
  JS compatible data types from WASM functions
  (published to npm, only meant to be consumed by JS lib)
- epubparse-js: JS library with ergonomic API including
  Typescript definitions
  (published to npm, for use in Browser and Node.js)

## Steps to release

### Prepare
- bump version in `epubparse-rs/Cargo.toml`
- bump versions in `epubparse-wasm/Cargo.toml` and `epubparse-wasm/package.json`
- go to `epubparse-wasm` folder and run `build_package.sh`
- bump version and `epubparse-wasm` dependency verion in `epubparse-js/package.json`
- commit

### Release
#### Crates.io
- `cd` into epubparse-rs
- run `cargo publish --dry-run` to verify 
- run `cargo publish`

#### NPM
##### Wasm
- `cd` into epubparse-wasm
- run `wasm-pack login`
- run `wasm-pack publish`

##### JS
- run `npm run build`
- run `npm publish`
