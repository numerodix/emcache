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

* 100,000x get (constant key): 26k/s  - 40k/s
* 100,000x set (constant key): 27k/s  - 44k/s
* 700,000x set w/ noreply (constant key): 87k/s  - 1225k/s
* 100,000x version: 37k/s  - 54k/s

emcache:

* 100,000x get (constant key): 18k/s  - 20k/s
* 100,000x set (constant key): 16k/s  - 31k/s
* 700,000x set w/ noreply (constant key): 97k/s  - 87k/s
* 100,000x version: 21k/s  - 33k/s


### Fill cache to a certain percentage

* using 4 client threads
* connecting to 127.0.0.1
* key: 10 printable chars, random
* value: 100-1000 bytes, random

memcached:

* 55k items/s - 31mb/s
* 198k items/s - 111mb/s  (pypy)

emcache:

* 50k items/s - 28mb/s
* 162k items/s - 90mb/s  (pypy)
