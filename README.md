# emcache

[![Build Status](https://travis-ci.org/numerodix/emcache.svg?branch=master)](https://travis-ci.org/numerodix/emcache)

An implementation of memcached in Rust.


## Features and todo list

* Implements the [memcached protocol](doc/Protocol-support.md).
* Bounded cache with LRU behavior.
* Concurrency model based on thread-per-connection.
* [Modular architecture](doc/Architecture.md). Transport layer is separate from storage and is configured in a N:1 topology with communication using immutable Cmd/Resp values over async channels.
* Fairly good test coverage.
* No config file, logging or daemonization yet.
* [Performance](pyemc/ABOUT.md) is generally within 0.5-1x of memcached.
* Currently (Dec 2015) only builds against rust-nightly due to linked-hash-map dependency.


## Development

To build:

    $ cargo build

To run unit tests:

    $ ./all_unit_tests.sh

To run the server:
    
    $ ./run_server.sh

The Python integration test suite supplies various forms of tests. To run them
against a running server:

    $ python -m pyemc.main -p $PORT            # integration tests
    $ python -m pyemc.main -p $PORT --stress   # burn in tests
    $ python -m pyemc.main -p $PORT --fill 45  # fill to 45% capacity
