import time

from pyemc.abstractions.test_api import TestCase


class TestStress(TestCase):
    def test_get_and_set_const_key_value(self):
        # establish connection
        self.client.get_stats()

        # measure set rate - noreply
        start_time = time.time()
        setn_num = 200000  # should take about 2secs
        for _ in range(setn_num):
            self.client.set('x', 'abc', noreply=True)
        end_time = time.time()
        setn_interval = end_time - start_time
        setn_rate = float(setn_num) / (setn_interval) * 2

        # measure set rate
        start_time = time.time()
        set_num = 50000  # should take about 2secs
        for _ in range(set_num):
            self.client.set('x', 'abc')
        end_time = time.time()
        set_interval = end_time - start_time
        set_rate = float(set_num) / (set_interval) * 2

        # measure get rate
        start_time = time.time()
        get_num = 50000  # should take about 2secs
        for _ in range(get_num):
            self.client.get('x')
        end_time = time.time()
        get_interval = end_time - start_time
        get_rate = float(get_num) / (get_interval) * 2

        # display results
        self.write("Made %d constant key set+noreply requests in %.2f seconds = %.2f requests/sec" %
                   (setn_num, setn_interval, setn_rate))
        self.write("Made %d constant key set requests in %.2f seconds = %.2f requests/sec" %
                   (set_num, set_interval, set_rate))
        self.write("Made %d constant key get requests in %.2f seconds = %.2f requests/sec" %
                   (get_num, get_interval, get_rate))
