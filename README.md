# Epubparse

[![NPM](https://img.shields.io/npm/v/epubparse-js)](https://www.npmjs.com/package/epubparse-js)
[![Crates.io](https://img.shields.io/crates/v/epubparse)](https://crates.io/crates/epubparse)
[![Docs.rs](https://img.shields.io/docsrs/epubparse)](https://docs.rs/epubparse/latest/epubparse/)

⚠️ This library is developed for my own very narrow use cases and probably insufficient for your needs.
It is published both as a Rust crate to crates.io and as a NPM package (ESM module) to npm.

The sole purpose of this project is essentially to convert Epub files to simple text-only 
`Book` structures, where a `Book` is a tree of `Chapter`s that contain text and/or subchapters. 
I am just using this to do different kinds of text analysis on a per chapter basis.

At the moment only epub files with epub2 compatible toc (table of content) files (.ncx) are supported.
Many epub3 files do contain epub2 toc files for compatibility reasons.
I do intend to implement support for epub3 toc files (.xhtml) in the future.

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
