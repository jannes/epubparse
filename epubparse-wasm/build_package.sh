#!/bin/bash
echo "make sure version numbers in Cargo.toml and package.json are the same!"
sleep 3
wasm-pack build
cp package.json pkg/
