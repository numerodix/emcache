from pyperf.abstractions.test_api import TestCase
from pyperf.client import ItemNotFoundError
from pyperf.client import SetFailedError
from pyperf.util import generate_random_data
from pyperf.util import generate_random_key


class TestApi(TestCase):
    def test_get_invalid_key(self):
        key = generate_random_key(4)

        self.write("Trying to get invalid key...")
        with self.assert_raises(ItemNotFoundError):
            item = self.client.get(key)

        self.write("...key not found")

    def test_set_and_get_large_value(self):
        key = generate_random_key(10)
        val = generate_random_data(1 << 19)  # .5mb

        self.write("Setting large value (%s):   %r -> %r..." % (len(val), key, val[:7]))
        self.client.set(key, val)

        item = self.client.get(key)
        val2 = item.value
        self.write("Retrieved large value (%s): %r -> %r..." % (len(val), key, val2[:7]))

        assert val == val2

    def test_set_and_get_multiple(self):
        key1 = 'a'
        val1 = '1'

        key2 = 'b'

        key3 = 'c'
        val3 = '3'

        self.write("Setting key %r -> %r" % (key1, val1))
        self.client.set(key1, val1)

        self.write("Setting key %r -> %r" % (key3, val3))
        self.client.set(key3, val3)

        keys = [key1, key2, key3]
        self.write("Getting keys %r" % keys)
        dct = self.client.get_multi(keys)
        self.write("Got keys: %r" % dct.keys())

        assert val1 == dct[key1].value
        assert val3 == dct[key3].value

    def test_set_and_get_small_key(self):
        key = generate_random_key(4)
        val = generate_random_data(5, 8)

        self.write("Setting small key:   %r -> %r" % (key, val))
        self.client.set(key, val)

        item = self.client.get(key)
        val2 = item.value
        self.write("Retrieved small key: %r -> %r" % (key, val2))

        assert val == val2, 'value read does not match value set'

    def test_set_too_large_key(self):
        key = generate_random_key(251)  # limit is 250b
        val = generate_random_data(1)

        self.write("Trying to set too large key (%s):   %r -> %r..." % (len(key), key[:7], val))
        with self.assert_raises(SetFailedError):
            self.client.set(key, val)

        self.write("...set failed")

    def test_set_too_large_value(self):
        key = generate_random_key(10)
        val = generate_random_data(1 << 21)  # 2mb, limit is 1mb

        self.write("Trying to set too large value (%s):   %r -> %r..." % (len(val), key, val[:7]))
        with self.assert_raises(SetFailedError):
            self.client.set(key, val)

        self.write("...set failed")
