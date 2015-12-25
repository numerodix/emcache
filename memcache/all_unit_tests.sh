#!/bin/bash

set -e

cargo test -- --ignored $@
./fasttests.sh $@
