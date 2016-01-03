#!/bin/bash

set -ex

# Options we'll use
PORT=11311

# Build the server
cargo build

# Launch the server, give it 5sec to start up
cargo run -- --port $PORT &
sleep 5

# Run the fill test
echo -e "\n=== FILL TEST ===\n"
python -m pyemc.main -p $PORT --fill 5.0

# Run the stress test
echo -e "\n=== STRESS TEST ===\n"
python -m pyemc.main -p $PORT --stress

# Run the integ tests
echo -e "\n=== INTEG TEST ===\n"
python -m pyemc.main -p $PORT
exit_code=$?

# Kill the server
ps axf | grep 'target\/.*\/emcache' | awk '{print $1}' | xargs kill || true

exit $exit_code
