#!/bin/bash

cargo build --release
cargo run -- $@
