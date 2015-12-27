Low hanging fruit:

* Cache capacity should be set on bytes, not items.
* Add cmd_get, cmd_set etc metrics in protocol.
* Add reclaimed, evicted etc metrics in storage.
* Get rid of AccountingHashMap - turns out to be of little use.
* Make transport parser more robust to invalid command strings - less panic prone.

Medium:

* Expose hardcoded constants for cache size, port number etc as cmdline args.

Large:

* Implement the full memcached protocol.
* Port process model from thread-per-connection w/channels to async with MetalIO.
