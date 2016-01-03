# emcache

[![Build Status](https://travis-ci.org/numerodix/emcache.svg?branch=master)](https://travis-ci.org/numerodix/emcache)

A toy implementation of memcached in Rust.


## Features and todo list

* Minimalistic implementation of the [memcached protocol](https://github.com/numerodix/emcache/blob/master/doc/Protocol-support.md).
* Bounded cache with LRU behavior.
* Cache item lifetime can be controlled both globally and on a per-item basis.
* Concurrency model based on thread-per-connection.
* [Modular architecture](https://github.com/numerodix/emcache/blob/master/doc/Architecture.md). Transport layer is separate from storage and is configured in a N:1 topology with communication using immutable Cmd/Resp values over async channels.
* Numerous opportunities for optimization by eliminating data copying.
* Fairly good test coverage.
* No config file, logging or daemonization yet.
* [Performance](https://github.com/numerodix/emcache/blob/master/pyperf/ABOUT.md) is roughly 1/3 of memcached (which uses async io).
* Currently (Dec 2015) only builds against rust-nightly due to linked-hash-map dependency.


## Development

To build:

    $ cargo build

To run unit tests:

    $ ./all_unit_tests.sh

To run the server:
    
    $ ./run_server.sh
