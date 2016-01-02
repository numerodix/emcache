#!/bin/bash

set -e

# Options we'll use
PORT=11311

# Build the server
cargo build --release

# Launch the server, give it 2sec to start up
cargo run -- --port $PORT >/dev/null &
sleep 2

# Run the fill test
echo -e "\n=== FILL TEST ===\n"
python -m pyperf.tester -p $PORT --fill 5.0

# Run the stress test
echo -e "\n=== STRESS TEST ===\n"
python -m pyperf.tester -p $PORT --stress

# Run the integ tests
echo -e "\n=== INTEG TEST ===\n"
python -m pyperf.tester -p $PORT
exit_code=$?

# Kill the server
ps axf | grep 'target\/.*\/memcache' | awk '{print $1}' | xargs kill || true

exit $exit_code
