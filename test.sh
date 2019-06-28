#!/usr/bin/env bash

set -e

cargo fmt
cargo clippy -- -Dwarnings
cargo test
