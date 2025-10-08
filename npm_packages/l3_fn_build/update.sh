#!/bin/sh

(cd ../.. && cargo build -p l3_fn_build_wasm --release --target wasm32-wasip2)

pnpm codegen
