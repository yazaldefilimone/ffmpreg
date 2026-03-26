#!/usr/bin/env bash

RUSTFLAGS="-Awarnings" cargo run --all-features --quiet -- $@
