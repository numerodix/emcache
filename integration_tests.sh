#!/bin/bash

set -e

# Build the server
cargo build

# Launch the server, give it 2sec to start up
./run_server.sh &
pid=$!
sleep 2

# Run the client tests
./client_test.py -p 11311
exit_code=$?

# Kill the server
kill -9 $pid

exit $exit_code
