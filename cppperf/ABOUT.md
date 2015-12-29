# Performance testing in C++

The idea was to write a highly optimal client to measure transaction rates both
against memcached and memcache.rs.

But during testing perftest runs at a higher cpu utilization than memcached, so
perftest seems unable to achieve a high enough request rate to measure the peak
rate for memcached.


## Measurements

Single threaded client, on the same machine.

* key: 'x'
* value: 'abc'

memcached:

* 10,000x get (constant key): 32k/s
* 10,000x set (constant key): 19k/s

memcache.rs:

* 10,000x get (constant key): 7.8k/s
* 10,000x set (constant key): 7.6k/s
