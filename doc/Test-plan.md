# Test plan


## Correctness

The primary vehicle for correctness testing is *unit tests*. Almost every
component in emcache is unit tested, including the storage, protocol and
transport.

The only exception is tasks, which occupy a thread and directly manipulate a
socket or channel, which makes them impractical to unit test. Evenso, they are
merely sequences of steps performed on underlying components which are
themselves unit tested.

As a practical matter, we try to avoid time based unit tests when possible and
mark tests that require sleeping as `ignore` so they are excluded from the
default unit test run. All unit tests must run in Travis CI.

We also use *integration tests* to verify that all the trusted pieces have been
wired up correctly. The integration tests must cover the entire protocol. When
extending integration tests the suite must also be run against memcached to
verify that the tests themselves work correctly.


## Performance

todo..
