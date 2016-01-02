# Performance testing in Python

This test suite includes a Task abstraction which allows defining a task as a
pair of Task (main job) + Tasklet (client run in seperate threads) to enable
parallelism.


## Measurements


### Set/get a constant key

* single threaded client
* connecting to 127.0.0.1
* key: 'x'
* value: 'abc'

memcached:

* 10,000x get (constant key): 67k/s
* 10,000x set (constant key): 65k/s

memcache.rs:

* 10,000x get (constant key): 49k/s
* 10,000x set (constant key): 41k/s


### Fill cache to a certain percentage

* using 4 client threads
* connecting to 127.0.0.1
* key: 10 printable chars, random
* value: 100-1000 bytes, random

memcached:

* 55k items/s - 31mb/s

memcache.rs:

* 50k items/s - 28mb/s
