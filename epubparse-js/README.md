# Epubparse-js

WORK IN PROGRESS

This library aims to convert Epub files into text-only Book structures
that can be used to do analysis of the contained text.

## Design goals
- perform a reasonable conversion into a book with chapters ✅
- support Epub version 2 ✅
- support Epub version 3 ❌ 
  (most version 3 epubs also include version 2 .ncx table of contents, these should also work)

## Development

Tests should be run with Node 16,
two experimental flags are used to deal with ESM modules and WASM:

- --experimental-specifier-resolution=node
- --experimental-wasm-modules