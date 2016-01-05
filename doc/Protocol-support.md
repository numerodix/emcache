# Protocol support

We follow [the official protocol specification](https://github.com/memcached/memcached/blob/master/doc/protocol.txt).


## Fully supported

* ADD
* APPEND
* DECR
* DELETE
* GET
* GETS
* INCR
* QUIT
* PREPEND
* REPLACE
* SET
* TOUCH
* VERSION


## Partial support

* FLUSH_ALL (without options)
* STATS (not all stats are present)


## Plan to support

* CAS


## No plan to support

These commands are too implementation specific to memcached itself for it to
make sense to support them.

* SLABS
* STATS [arg]
* VERBOSITY
