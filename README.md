# memcache.rs

[![Build Status](https://travis-ci.org/numerodix/memcache.rs.svg?branch=master)](https://travis-ci.org/numerodix/memcache.rs)

A toy implementation of memcached in Rust.


## Features and todo list

* Minimalistic implementation of GET, SET and STATS commands.
* Bounded cache with LRU behavior.
* Cache item lifetime can be controlled both globally and on a per-item basis.
* Concurrency model based on thread-per-connection.
* [Modular architecture](https://github.com/numerodix/memcache.rs/blob/master/doc/Architecture.md). Transport layer is separate from storage and is configured in a N:1 topology with communication using immutable Cmd/Resp values over async channels.
* Numerous opportunities for optimization by eliminating data copying.
* Fairly good test coverage.
* No config file, logging or daemonization yet.
* Currently (Dec 2015) only builds against rust-nightly due to linked-hash-map dependency.


## Development

To build:

    $ cargo build

To run unit tests:

    $ ./all_unit_tests.sh

To run the server:
    
    $ ./run_server.sh
