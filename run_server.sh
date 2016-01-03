#!/bin/bash

cargo build --release
target/release/emcache $@
