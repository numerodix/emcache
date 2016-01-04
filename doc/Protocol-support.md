# Protocol support

We follow [the official protocol specification](https://github.com/memcached/memcached/blob/master/doc/protocol.txt).


## Fully supported

* ADD
* APPEND
* DELETE
* GET
* QUIT
* REPLACE
* SET
* VERSION


## Partial support

* STATS (not all stats are present)


## Plan to support

* PREPEND
* CAS/GETS
* FLUSH_ALL
* INCR/DECR
* TOUCH


## No plan to support

These commands are too implementation specific to memcached itself for it to
make sense to support them.

* SLABS
* STATS [arg]
* VERBOSITY
