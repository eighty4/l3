#!/bin/sh
set -e

# run through all the checks done for ci

cargo fmt -v
cargo clippy -- -D warnings
cargo build
cargo test
