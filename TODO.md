Low hanging fruit:

* ...

Medium:

* Add a logging facility and start adding some basic log output.
* Add fuzzing test to pyperf by supplying valid samples of command strings and randomly shuffling characters / shortening/elongating fields.
* Smoke testing using python-memcache (third party client) to establish protocol correctness.

Large:

* Implement the full memcached protocol.
* Port process model from thread-per-connection w/channels to async with MetalIO.
