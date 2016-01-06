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

* Against commit d81b62a (Jan 6, 2016)
* Using 4 client threads
* Connecting to 127.0.0.1
* Cache size: 512mb
* Rate is averaged over the whole run (measuring items/s and bytes/s)

Fill mode:

* key: 10 printable chars, random
* value: 100-1000 bytes, random
* set w/noreply and pipelined in batches of 100

| Task              | memcached/cpython     | emcache/cpython   | memcached/pypy   | emcache/pypy         |
|-------------------|----------------------:|------------------:|-----------------:|---------------------:|
| Fill cache to 80% | **567k/s - 317mb/s**  | 503k/s - 281mb/s  | 594k/s - 332mb/s | **614k/s - 343mb/s** |


### Memory efficiency - per commit 8741b0b (Jan 6, 2016)

Ideally, emcache would use only as much memory as the user actually stores. In
practice, there is a certain overhead because we also need various
datastructures, and each transport needs a few buffers to operate.

If we look at storage alone, there is a certain overhead to storing keys (storing the vector is 24 bytes on 64bit) and values (vector + flags + exptime + atime + cas_unique).

* A key is currently 24 bytes + the key data.
* A value is currently 56 bytes + the value data.

For really small keys (ints), the overhead completely dominates the user data.

For the fill test we see these numbers:

* Cache capacity is 1024mb
* Cache utilization is 80%

| User stored size | emcache bytes stat | process residental size |
|-----------------:|-------------------:|------------------------:|
| 716mb            | 819mb (87% eff.)   | 993mb (72% eff.)        |
