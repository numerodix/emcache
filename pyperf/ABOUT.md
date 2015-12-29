# Performance testing in Python

This test suite includes a Task abstraction which allows defining a task as a
pair of Task (main job) + Tasklet (client run in seperate threads) to enable
parallelism.

The current bottleneck in pyperf is the inability to generate random keys fast
enough to push the transaction rate higher than it currently goes.


## Measurements


### Set/get a constant key

Single threaded client, on the same machine.

* key: 'x'
* value: 'abc'

memcached:

* 10,000x get (constant key): 67k/s
* 10,000x set (constant key): 65k/s

memcache.rs:

* 10,000x get (constant key): 15.9k/s
* 10,000x set (constant key): 15.6k/s


### Fill cache to a certain percentage

Using 4 threads.

* key: 10 printable chars, random
* value: 100-1000 bytes, random

memcached:

* 10.2k items/s - 6.0mb/s

memcache.rs:

* 2.2k items/s - 1.2mb/s
