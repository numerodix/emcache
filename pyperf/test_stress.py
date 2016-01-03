import time

from pyperf.abstractions.test_api import TestCase


class TestStress(TestCase):
    def test_get_and_set_const_key_value(self):
        # establish connection
        self.client.get_stats()

        # measure set rate
        start_time = time.time()
        num = 50000  # should take about 2secs
        for _ in range(num):
            self.client.set('x', 'abc', noreply=True)
        end_time = time.time()
        set_interval = end_time - start_time
        set_rate = float(num) / (set_interval) * 2

        # measure set rate
        start_time = time.time()
        num = 50000  # should take about 2secs
        for _ in range(num):
            self.client.get('x')
        end_time = time.time()
        get_interval = end_time - start_time
        get_rate = float(num) / (get_interval) * 2

        # display results
        self.write("Made %d constant key set requests in %.2f seconds = %.2f requests/sec" %
                   (num, set_interval, set_rate))
        self.write("Made %d constant key get requests in %.2f seconds = %.2f requests/sec" %
                   (num, get_interval, get_rate))
