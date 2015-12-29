# Performance testing in Python

This test suite includes a Task abstraction which allows defining a task as a
pair of Task (main job) + Tasklet (client run in seperate threads) to enable
parallelism.


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

* 55k items/s - 31mb/s

memcache.rs:

* 3k items/s - 1.7mb/s
