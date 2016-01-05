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

| Test                   | memcached/cpython | emcache/cpython | memcached/pypy | emcache/pypy |
|------------------------|------------------:|----------------:|---------------:|-------------:|
| 100,000x get           | **26k/s**         | 18k/s           | **40k/s**      | 20k/s        |
| 100,000x set           | **27k/s**         | 16k/s           | **44k/s**      | 31k/s        |
| 700,000x set w/noreply | 87k/s             | **97k/s**       | **1225k/s**    | 87k/s        |
| 100,000x version       | **37k/s**         | 21k/s           | **54k/s**      | 33k/s        |

The pypy version was run 10x longer to account for jit warmup, but the rate is
still averaged over the whole run.


### Fill cache to a certain percentage

* using 4 client threads
* connecting to 127.0.0.1
* key: 10 printable chars, random
* value: 100-1000 bytes, random
* 512mb size cache
* rate averaged over the whole run

| Task              | memcached/cpython   | emcache/cpython | memcached/pypy        | emcache/pypy    |
|-------------------|--------------------:|----------------:|----------------------:|----------------:|
| Fill cache to 80% | **55k/s - 31mb/s**  | 50k/s - 28mb/s  | **198k/s - 111mb/s**  | 162k/s - 90mb/s |
