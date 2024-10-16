# Epubparse

[![NPM](https://img.shields.io/npm/v/epubparse-js)](https://www.npmjs.com/package/epubparse-js)

⚠️ This library is developed for my own very narrow use cases and probably insufficient for your needs.
It is published both as a Rust crate to crates.io and as a NPM package (ESM module) to npm.

The sole purpose of this project is essentially to convert Epub files to simple text-only 
`Book` structures, where a `Book` is a tree of `Chapter`s that contain text and/or subchapters. 
I am just using this to do different kinds of text analysis on a per chapter basis.

At the moment only epub files with epub2 compatible toc (table of content) files (.ncx) are supported.
Many epub3 files do contain epub2 toc files for compatibility reasons.
I do intend to implement support for epub3 toc files (.xhtml) in the future.
