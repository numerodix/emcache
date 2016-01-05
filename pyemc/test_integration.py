import math
import time

from pyemc.abstractions.test_api import TestCase
from pyemc.client import ClientError
from pyemc.client import DeleteFailedError
from pyemc.client import NotFoundError
from pyemc.client import NotStoredError
from pyemc.client import ServerError
from pyemc.util import generate_random_data
from pyemc.util import generate_random_key


class TestApi(TestCase):

    ## Happy path
    
    # Add

    def test_add(self):
        key = generate_random_key(8)
        val = generate_random_data(10)

        self.client.add(key, val)
        item = self.client.get(key)
        assert val == item.value

        # try to add an existing key
        with self.assert_raises(NotStoredError):
            self.client.add(key, val)

    def test_add_noreply(self):
        key = generate_random_key(8)
        val = generate_random_data(10)

        self.client.add(key, val, noreply=True)
        item = self.client.get(key)
        assert val == item.value

    # Append

    def test_append(self):
        key = generate_random_key(8)
        val = generate_random_data(10)
        val2 = generate_random_data(10)

        # try to append to an invalid key
        with self.assert_raises(NotStoredError):
            self.client.append(key, val)

        self.client.set(key, val)
        self.client.append(key, val2)
        item = self.client.get(key)
        assert val + val2 == item.value

    def test_append_noreply(self):
        key = generate_random_key(8)
        val = generate_random_data(10)
        val2 = generate_random_data(10)

        self.client.set(key, val)
        self.client.append(key, val2, noreply=True)
        item = self.client.get(key)
        assert val + val2 == item.value

    # Decr

    def test_decr(self):
        key = generate_random_key(10)
        val = '1'

        # try to decr an invalid key
        with self.assert_raises(NotFoundError):
            self.client.decr(key)

        self.client.set(key, val)
        val2 = self.client.decr(key, '1')
        assert int(val) - 1 == int(val2)

    def test_decr_noreply(self):
        key = generate_random_key(10)
        val = '1'

        self.client.set(key, val)
        self.client.decr(key, noreply=True)
        item = self.client.get(key)
        assert int(val) - 1 == int(item.value)

    # Delete

    def test_delete(self):
        key = generate_random_key(8)
        val = generate_random_data(10)

        # try to delete invalid key
        with self.assert_raises(NotFoundError):
            self.client.delete(key)

        self.client.set(key, val)
        self.client.delete(key)
        with self.assert_raises(NotFoundError):
            self.client.get(key)

    def test_delete_noreply(self):
        key = generate_random_key(8)
        val = generate_random_data(10)

        self.client.set(key, val)
        self.client.delete(key, noreply=True)
        with self.assert_raises(NotFoundError):
            self.client.get(key)

    # FlushAll

    def test_flush_all(self):
        key = generate_random_key(4)
        val = generate_random_data(5, 8)

        # key set before flush is expired
        self.client.set(key, val)
        self.client.flush_all()
        with self.assert_raises(NotFoundError):
            self.client.get(key)

        # sleep a bit to make sure we don't get any rounding errors on the
        # exact flush timestamp
        time.sleep(1.5)

        # key set after flush works as expected
        key2 = generate_random_key(4)
        val2 = generate_random_data(5, 8)
        self.client.set(key2, val2)
        item = self.client.get(key2)
        assert item.value == val2

    # Get and Set

    def test_set_and_get_small_key(self):
        key = generate_random_key(4)
        val = generate_random_data(5, 8)

        self.client.set(key, val)

        item = self.client.get(key)
        val2 = item.value

        assert val == val2

    def test_set_and_get_large_value(self):
        key = generate_random_key(10)
        val = generate_random_data(1 << 19)  # .5mb

        self.client.set(key, val)

        item = self.client.get(key)
        val2 = item.value

        assert val == val2

    def test_get_multiple(self):
        key1 = generate_random_key(10)
        val1 = generate_random_data(10)

        key2 = generate_random_key(10)

        key3 = generate_random_key(10)
        val3 = generate_random_data(10)

        self.client.set(key1, val1)

        self.client.set(key3, val3)

        keys = [key1, key2, key3]
        dct = self.client.get_multi(keys)

        assert val1 == dct[key1].value
        assert val3 == dct[key3].value

    def test_set_exptime_abs_2s(self):
        key = generate_random_key(10)
        val = generate_random_data(10)

        # we don't know if we have time sync with the server, so fetch the
        # server's time first
        stats = self.client.get_stats()
        now = int(stats['time'])

        self.client.set(key, val, exptime=now + 1)
        item = self.client.get(key)  # still there

        time.sleep(2.3)
        with self.assert_raises(NotFoundError):
            item = self.client.get(key)  # expired

    def test_set_exptime_rel_1s(self):
        key = generate_random_key(10)
        val = generate_random_data(10)

        self.client.set(key, val, exptime=1)
        item = self.client.get(key)  # still there

        time.sleep(1.1)
        with self.assert_raises(NotFoundError):
            item = self.client.get(key)  # expired

    def test_set_flags(self):
        key = generate_random_key(10)
        val = generate_random_data(10)
        flags = 15

        self.client.set(key, val, flags=flags)
        item = self.client.get(key)

        flags2 = item.flags
        val2 = item.value

        assert val == val2
        assert flags == flags2

    def test_set_noreply(self):
        key = generate_random_key(10)
        val = generate_random_data(10)

        # set without requesting confirmation
        self.client.set(key, val, noreply=True)

        # verify that it worked
        item = self.client.get(key)
        val2 = item.value

        assert val == val2

    # Gets

    def test_gets(self):
        key = generate_random_key(8)
        val = generate_random_data(10)

        # set a key, record cas_unique
        self.client.set(key, val)
        item = self.client.gets(key)

        # set again, cas_unique should have changed
        val2 = generate_random_data(10)
        self.client.set(key, val)
        item2 = self.client.gets(key)
        assert item.cas_unique != item2.cas_unique

    def test_gets_multi(self):
        key1 = generate_random_key(8)
        val1 = generate_random_data(10)
        key2 = generate_random_key(8)
        val2 = generate_random_data(10)

        # i can fetch values as normal, cas_unique is set
        self.client.set(key1, val1)
        self.client.set(key2, val2)
        dct = self.client.gets_multi([key1, key2])
        assert dct[key1].value == val1
        assert dct[key2].value == val2
        assert dct[key1].cas_unique is not None
        assert dct[key2].cas_unique is not None

    # Incr

    def test_incr(self):
        key = generate_random_key(10)
        val = '1'

        # try to incr an invalid key
        with self.assert_raises(NotFoundError):
            self.client.incr(key)

        self.client.set(key, val)
        val2 = self.client.incr(key, '40')
        assert int(val) + 40 == int(val2)

    def test_incr_noreply(self):
        key = generate_random_key(10)
        val = '1'

        self.client.set(key, val)
        self.client.incr(key, noreply=True)
        item = self.client.get(key)
        assert int(val) + 1 == int(item.value)

    # Quit

    def test_quit(self):
        self.client.quit()

    # Prepend

    def test_prepend(self):
        key = generate_random_key(8)
        val = generate_random_data(10)
        val2 = generate_random_data(10)

        # try to prepend to an invalid key
        with self.assert_raises(NotStoredError):
            self.client.prepend(key, val)

        self.client.set(key, val)
        self.client.prepend(key, val2)
        item = self.client.get(key)
        assert val2 + val == item.value

    def test_prepend_noreply(self):
        key = generate_random_key(8)
        val = generate_random_data(10)
        val2 = generate_random_data(10)

        self.client.set(key, val)
        self.client.prepend(key, val2, noreply=True)
        item = self.client.get(key)
        assert val2 + val == item.value

    # Replace

    def test_replace(self):
        key = generate_random_key(8)
        val = generate_random_data(10)
        val2 = generate_random_data(10)

        # try to replace an invalid key
        with self.assert_raises(NotStoredError):
            self.client.replace(key, val)

        self.client.set(key, val)
        self.client.replace(key, val2)
        item = self.client.get(key)
        assert val2 == item.value

    def test_replace_noreply(self):
        key = generate_random_key(8)
        val = generate_random_data(10)
        val2 = generate_random_data(10)

        self.client.set(key, val)
        self.client.replace(key, val2, noreply=True)
        item = self.client.get(key)
        assert val2 == item.value

    # Stats

    def test_get_stats(self):
        dct = self.client.get_stats()
        for (key, value) in dct.items():
            self.write('%s: %s' % (key, value))

    # Touch

    def test_touch(self):
        key = generate_random_key(8)
        val = generate_random_data(10)

        # try to touch an invalid key
        with self.assert_raises(NotFoundError):
            self.client.touch(key)

        # expires in 3s
        self.client.set(key, val, exptime=3)

        time.sleep(1.5)

        # keep it alive another 3s
        # TODO: what should happen if exptime is unset?
        self.client.touch(key, exptime=3)

        time.sleep(1.5)

        item = self.client.get(key)
        assert val == item.value

    def test_touch_noreply(self):
        key = generate_random_key(8)
        val = generate_random_data(10)

        # expires in 3s
        self.client.set(key, val, exptime=3)

        time.sleep(1.5)

        # keep it alive another 3s
        self.client.touch(key, exptime=3, noreply=True)

        time.sleep(1.5)

        item = self.client.get(key)
        assert val == item.value

    # Version

    def test_version(self):
        version = self.client.version()
        self.write(version)


    ## Failure cases

    def test_get_invalid_key(self):
        key = generate_random_key(10)

        self.client.delete(key, noreply=True)
        with self.assert_raises(NotFoundError):
            item = self.client.get(key)

    def test_decr_underflow(self):
        key = generate_random_key(10)
        val = '0'

        self.client.set(key, val)
        val2 = self.client.decr(key)
        assert int(val) == int(val2)

    def test_incr_overflow(self):
        key = generate_random_key(10)
        val = str((1 << 64) - 1)

        # set max unsigned 64bit value - overflows to 0
        self.client.set(key, val)
        val2 = self.client.incr(key)
        assert val2 == str(0)

    def test_incr_over_size(self):
        key = generate_random_key(10)
        val = str(1 << 64)  # cannot store in 64 bits

        self.client.set(key, val)
        with self.assert_raises(ClientError):
            self.client.incr(key)  # not treated as a number


    ## Exceed limits
    # TODO try key/val too large for each command?

    def test_set_too_large_key(self):
        key = generate_random_key(251)  # limit is 250b
        val = generate_random_data(1)

        with self.assert_raises(ClientError):
            self.client.set(key, val)

    def test_set_too_large_value(self):
        key = generate_random_key(10)
        val = generate_random_data(1 << 21)  # 2mb, limit is 1mb

        with self.assert_raises(ServerError):
            self.client.set(key, val)
