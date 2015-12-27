Low hanging fruit:

* Cache capacity should be set on bytes, not items.
* Add reclaimed, evicted etc metrics in storage.
* Add bytes_read, bytes_written metrics to transport and transmit to driver.
* Make transport parser more robust to invalid command strings - less panic prone.

Medium:

* Expose hardcoded constants for cache size, port number etc as cmdline args.
* Add a logging facility and start adding some basic log output.

Large:

* Implement the full memcached protocol.
* Port process model from thread-per-connection w/channels to async with MetalIO.
* Create a test harness to collect performance metrics under concurrent execution.
