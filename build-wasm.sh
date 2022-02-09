#!/bin/bash

wasm-pack build --release --target web
wasm-opt -Os -o pkg/cnccoder_bg.wasm pkg/cnccoder_bg.wasm
