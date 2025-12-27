#!/usr/bin/env sh

set -eux

cargo fmt
cargo check
cargo clippy
cargo test
