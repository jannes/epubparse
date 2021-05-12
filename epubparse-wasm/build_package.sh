#!/bin/bash
echo "make sure installed wasm-pack is from master branch, latest release (0.9.1) is broken"
# https://github.com/rustwasm/wasm-pack/issues/837
echo "make sure version numbers in Cargo.toml and package.json are the same!"
sleep 3
wasm-pack build
cp package.json pkg/
