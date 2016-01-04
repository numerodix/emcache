import time

from pyemc.abstractions.test_api import TestCase


class TestStress(TestCase):
    def run_bench(self, func, loops, desc):
        # untimed warmup
        warmup_loops = loops / 3
        for _ in xrange(loops):
            func()

        # timed execution
        start_time = time.time()
        for _ in xrange(loops):
            func()
        end_time = time.time()
        interval = end_time - start_time

        rate = float(loops) / interval

        self.write("Made %d %s requests in %.2f seconds = %.2f requests/sec" %
                   (loops, desc, interval, rate))

    def test_set_const_key_noreply(self):
        def func():
            self.client.set('x', 'abc', noreply=True)

        self.run_bench(func, 700000, 'constant key set+noreply')

    def test_set_const_key(self):
        def func():
            self.client.set('x', 'abc')

        self.run_bench(func, 100000, 'constant key set')

    def test_get_const_key(self):
        self.client.set('x', 'abc')

        def func():
            self.client.get('x')

        self.run_bench(func, 100000, 'constant key get')

    def test_version(self):
        '''Does not even touch the storage layer.'''
        def func():
            self.client.version()

        self.run_bench(func, 100000, 'version')
