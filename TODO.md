Low hanging fruit:

* ...

Medium:

* Expose hardcoded constants for cache size, port number etc as cmdline args.
* Add a logging facility and start adding some basic log output.

Large:

* Implement the full memcached protocol.
* Port process model from thread-per-connection w/channels to async with MetalIO.
* Create a test harness to collect performance metrics under concurrent execution.
