# Epubparse

WORK IN PROGRESS
see epubparse-rs/README.md for project motivation/goals

## Structure
- epubparse-rs: core Rust library that compiles to WASM
  (published to crates.io)
- epubparse-wasm: wrapper around Rust core that provides
  JS compatible data types from WASM functions
  (published to npm, only meant to be consumed by JS lib)
- epubparse-js: JS library with ergonomic API including
  Typescript definitions
  (published to npm, for use in Browser and Node.js)
