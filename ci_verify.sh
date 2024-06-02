#!/bin/sh
set -e

# run through all the checks done for ci

if [ -n "$(git status --porcelain)" ]; then
  echo "error: run \`ci_verify.sh\` with a clean working directory"
  exit 1
fi

cargo fmt -v
git diff --exit-code
cargo clippy -- -D warnings
cargo build
cargo test
