#!/bin/bash

set -e

# Build the server
cargo build

# Launch the server, give it 2sec to start up
./run_server.sh >/dev/null &
sleep 2

# Run the fill test
echo -e "\n=== FILL TEST ===\n"
python -m pyperf.tester -p 11311 --fill 5.0

# Run the stress test
echo -e "\n=== STRESS TEST ===\n"
python -m pyperf.tester -p 11311 --stress

# Run the integ tests
echo -e "\n=== INTEG TEST ===\n"
python -m pyperf.tester -p 11311
exit_code=$?

# Kill the server
ps axf | grep target/*/memcache | awk '{print $1}' | xargs kill || true

exit $exit_code
