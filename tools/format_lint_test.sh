#!/usr/bin/env sh

set -eux

cargo check
cargo clippy
cargo test
cargo fmt
