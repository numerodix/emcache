import time
import threading

from perftest.util import generate_random_key
from perftest.util import generate_random_data
from perftest.util import insert_number_commas


class LoadGenerator(object):
    def __init__(self, client_params):
        self.client_params = client_params

    def fill_to_pct(self, percentage):
        threads = []
        for i in range(5):
            t = threading.Thread(target=self._fill_to_pct, args=(percentage,))
            threads.append(t)
            t.start()

    def _fill_to_pct(self, percentage):
        '''
        Fill the cache with random data to be approx x% full.
        TODO: Address key/value size distribution
        '''

        client = self.client_params.create_client()

        stats = client.get_stats()
        capacity = stats['limit_maxbytes']
        capacity_fmt = insert_number_commas(capacity)

        def get_pct_full():
            stats = client.get_stats()
            capacity = stats['limit_maxbytes']
            bytes = stats['bytes']
            pct_full = 100 * float(bytes) / float(capacity)
            return pct_full

        pct_full = get_pct_full()
        batch_size = 1000
        time_cum = 0
        items_cum = 0
        rate = -1

        print("Filling to ~%s%% (of %s)" % (percentage, capacity_fmt))

        while pct_full < percentage:
            print("Cache is %.2f%% full of %s, inserting %s items (rate: %.2f items/s)" %
                  (pct_full, capacity_fmt, batch_size, rate))

            time_st = time.time()

            for _ in range(1000):
                key = generate_random_key(10)
                value = generate_random_data(100, 1000)
                client.set(key, value)

            duration = time.time() - time_st
            rate = batch_size / duration
            time_cum += duration
            items_cum += batch_size

            pct_full = get_pct_full()

        rate_cum = items_cum / time_cum if time_cum > 0 else 0
        print("Done filling, took %.2fs to insert %s items (avg rate: %.2f items/s)" %
              (time_cum, items_cum, rate_cum))
