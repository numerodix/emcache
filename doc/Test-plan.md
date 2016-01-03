# Test plan


## Correctness

The primary vehicle for correctness testing is **unit tests**. Almost every
component in emcache is unit tested, including the storage, protocol and
transport.

The only exception is tasks, which occupy a thread and directly manipulate a
socket or channel, which makes them impractical to unit test. Evenso, they are
merely sequences of steps performed on underlying components which are
themselves unit tested.

As a practical matter, we try to avoid time based unit tests when possible and
mark tests that require sleeping as `ignore` so they are excluded from the
default unit test run. All unit tests must run in Travis CI.

We also use **integration tests** to verify that all the trusted pieces have been
wired up correctly. The integration tests must cover the entire protocol. When
extending integration tests the suite must also be run against memcached to
verify that the tests themselves work correctly.


## Performance

Performance testing of memcached, which is essentially an in-memory hashtable
exposed on a socket, is a tricky dance, given just how performant the
storage layer is.

**Rust level mirco benchmarks** show that setting or getting a key runs on the
order of 200 nanoseconds. So theoretically we can store 5M keys per second.
Even tacking on a protocol layer only takes it up to about 300-400ns.

And yet **tcp based stress testing** over 127.0.0.1 shows a much lower rate of
300k/s for key insertion in `noreply` mode where the client just spits out set
commands without receiving an acknowledgment, and an even lower rate of 30k
without `noreply` when each request is followed by a response before making the
next one.

So clearly the bottleneck is TCP, and the game is won or lost by making the
transport layer (in emcache terms) as efficient as possible. This is fair game
for the server, but it is unfortunate for the client generating this traffic,
which inescapably too is saddled with this responsibility. It bears keeping
in mind, then, that a performance test can only go as high as the client
itself can go, and at the end of the day we are merely writing items into a
hashmap.

...todo...

*NOTE:* All the numbers cited in this section are relative to one single
machine, so numbers will be different between machines (obviously).
