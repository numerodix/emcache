#!/bin/bash

ARG=$1; shift;

if [ "$ARG" == "--macros" ]; then
    rustc -Z unstable-options --test --pretty expanded src/main.rs
else
    cargo run
fi
