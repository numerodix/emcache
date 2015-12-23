#!/bin/bash

cargo test -- --ignored $@ && \
./fasttests.sh
