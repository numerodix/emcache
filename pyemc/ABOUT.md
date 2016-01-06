# Performance testing in Python

This test suite includes a Task abstraction which allows defining a task as a
pair of Task (main job) + Tasklet (client run in seperate threads) to enable
parallelism.


## Measurements


### Set/get a constant key

* against commit 67c5cda (Jan 5, 2016)
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

Test parameters:

* Against commit 244c8f5 (Jan 6, 2016)
* Using 4 client threads
* Connecting to 127.0.0.1
* Cache size: 512mb
* Rate is averaged over the whole run

Fill mode:

* key: 10 printable chars, random
* value: 100-1000 bytes, random
* set w/noreply and pipelined in batches of 100

| Task              | memcached/cpython     | emcache/cpython   | memcached/pypy   | emcache/pypy         |
|-------------------|----------------------:|------------------:|-----------------:|---------------------:|
| Fill cache to 80% | **490k/s - 274mb/s**  | 489k/s - 273mb/s  | 1.1m/s - 621mb/s | **1.1m/s - 636mb/s** |
