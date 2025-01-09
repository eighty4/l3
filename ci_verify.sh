#!/bin/sh
set -e

# run through all the checks done for ci

_git_status_output=$(git status --porcelain)

echo '\n*** cargo build ***'
cargo build --workspace --exclude l3_fn_build_wasm
cargo build -p l3_fn_build_wasm --target wasm32-wasip2

echo '\n*** cargo fmt -v ***'
cargo fmt --all -v
if [ -z "$_git_status_output" ]; then
  git diff --exit-code
fi

echo '\n*** cargo test ***'
cargo test --workspace

echo '\n*** cargo clippy -- -D warnings ***'
cargo clippy --all -- -D warnings

echo '\n*** cargo run --example(s) ***'
(cd fn_build && cargo run --example build_fn)
(cd fn_build && cargo run --example parse_fn)

if [ -n "$_git_status_output" ]; then
  echo
  echo "all ci verifications passed"
  echo "however, working directory had uncommited changes before running cargo fmt"
  exit 1
fi
