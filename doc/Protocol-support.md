# Protocol support

We follow [the official protocol specification](https://github.com/memcached/memcached/blob/master/doc/protocol.txt).


## Fully supported

* DELETE
* GET
* SET


## Partial support

* STATS (not all stats are present)


## Plan to support

* ADD/REPLACE
* APPEND/PREPEND
* CAS
* FLUSH_ALL
* GETS
* INCR/DECR
* QUIT
* TOUCH
* VERSION


## No plan to support

These commands are too implementation specific to memcached itself for it to
make sense to support them.

* SLABS
* STATS [arg]
* VERBOSITY
