#!/bin/bash

set -e

cargo test -- --ignored $@
./fast_unit_tests.sh $@
