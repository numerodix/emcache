from itertools import izip
import time

from pyemc.abstractions.task_api import Task
from pyemc.abstractions.task_api import Tasklet
from pyemc.util import generate_random_data
from pyemc.util import generate_random_data_prng
from pyemc.util import generate_random_key
from pyemc.util import generate_random_key_uuid
from pyemc.util import insert_number_commas


class CacheFillerTask(Task):
    def __init__(self, percentage=None, jobs=None, *args, **kwargs):
        super(CacheFillerTask, self).__init__(*args, **kwargs)
        self.percentage = percentage or 0
        self.jobs = jobs or 4

    def create_tasklets(self, state):
        tasklets = []

        for i in range(self.jobs):
            tasklet = CacheFillerTasklet(
                id=i + 1,
                client_params=self.client_params,
                percentage=self.percentage,
            )
            tasklets.append(tasklet)

        return tasklets

    def pre_tasklets(self, client, state):
        stats = client.get_stats()
        capacity = stats['limit_maxbytes']
        capacity_fmt = insert_number_commas(capacity)

        state.time_start = time.time()
        self.write("Filling to ~%s%% (of %s)" % (self.percentage, capacity_fmt))

    def post_tasklets(self, client, state, metrics_list):
        state.time_stop = time.time()
        state.duration = state.time_stop - state.time_start

        time_cum = sum([m.time_cum for m in metrics_list])
        time_total_cum = sum([m.time_total_cum for m in metrics_list])
        overhead_pct = 100 * (time_total_cum - time_cum) / time_total_cum if time_total_cum else 0

        items_cum = sum([m.items_cum for m in metrics_list])
        items_cum_str = insert_number_commas(str(items_cum))
        rate_items_cum = float(items_cum) / float(time_cum) if time_cum > 0 else 0
        rate_items_str = insert_number_commas(str(int(rate_items_cum)))

        bytes_cum = sum([m.bytes_cum for m in metrics_list])
        bytes_cum_str = insert_number_commas(str(bytes_cum))
        rate_bytes_cum = float(bytes_cum) / float(time_cum) if time_cum > 0 else 0
        rate_bytes_str = insert_number_commas(str(int(rate_bytes_cum)))

        self.write("Done filling, took %.2fs to insert %s items"
                   " (net avg rate: %s items/s - %s bytes/s)" %
                   (state.duration, items_cum_str, rate_items_str, rate_bytes_str))
        self.write("Spent %.2fs in network io, %.2fs in total (%.2f%% overhead - data gen, thread scheduling)"
                   % (time_cum, time_total_cum, overhead_pct))
        self.write("Wrote %s bytes in total" % bytes_cum_str)


class CacheFillerTasklet(Tasklet):
    def __init__(self, percentage=0, *args, **kwargs):
        super(CacheFillerTasklet, self).__init__(*args, **kwargs)
        self.percentage = percentage

    def get_pct_full(self, client):
        stats = client.get_stats()
        capacity = stats['limit_maxbytes']
        bytes = stats['bytes']
        pct_full = 100 * float(bytes) / float(capacity)
        return pct_full

    def run(self, client, metrics):
        # use client.set + client.flush_pipeline pattern
        client.pipeline_mode = True

        stats = client.get_stats()
        capacity = stats['limit_maxbytes']
        capacity_fmt = insert_number_commas(capacity)

        metrics.pct_full = self.get_pct_full(client)
        metrics.batch_size = 100
        metrics.bytes_cum = 0
        metrics.time_cum = 0
        metrics.time_total_cum = 0
        metrics.items_cum = 0
        rate = -1

        time_total_st = time.time()

        while metrics.pct_full < self.percentage:
            self.write("Cache is %.2f%% full of %s, inserting %s items (rate: %s items/s)" %
                       (metrics.pct_full, capacity_fmt, metrics.batch_size,
                        insert_number_commas(str(int(rate)))))

            # Pre-generate keys and values to avoid timing this work
            # TODO allow tuning the sizes of keys and values
            keys = [generate_random_key_uuid(10)
                    for _ in xrange(metrics.batch_size)]
            values = [generate_random_data(100, 1000)
                      for _ in xrange(metrics.batch_size)]

            for key, value in izip(keys, values):
                if not self._runnable:
                    return

                client.set(key, value, noreply=True)

                metrics.bytes_cum += len(key) + len(value)

            time_st = time.time()
            client.flush_pipeline()
            duration = time.time() - time_st

            rate = metrics.batch_size / duration
            metrics.time_cum += duration
            metrics.items_cum += metrics.batch_size

            metrics.pct_full = self.get_pct_full(client)
            metrics.time_total_cum = time.time() - time_total_st
