import time

from pyperf.task_api import Task
from pyperf.task_api import Tasklet
from pyperf.util import generate_random_data
from pyperf.util import generate_random_key
from pyperf.util import generate_random_key_uuid
from pyperf.util import insert_number_commas


class CacheFillerTask(Task):
    def __init__(self, percentage=0, jobs=4, *args, **kwargs):
        super(CacheFillerTask, self).__init__(*args, **kwargs)
        self.percentage = percentage
        self.jobs = jobs

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

        items_cum = sum([m.items_cum for m in metrics_list])
        rate_items_cum = float(items_cum) / float(time_cum) if time_cum > 0 else 0
        rate_items_cum = rate_items_cum * len(metrics_list)
        rate_items_str = insert_number_commas(str(int(rate_items_cum)))

        bytes_cum = sum([m.bytes_cum for m in metrics_list])
        rate_bytes_cum = float(bytes_cum) / float(time_cum) if time_cum > 0 else 0
        rate_bytes_cum = rate_bytes_cum * len(metrics_list)
        rate_bytes_str = insert_number_commas(str(int(rate_bytes_cum)))

        self.write("Done filling, took %.2fs to insert %s items"
                   " (avg rate: %s items/s - %s bytes/s)" %
                   (state.duration, items_cum, rate_items_str, rate_bytes_str))


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
        stats = client.get_stats()
        capacity = stats['limit_maxbytes']
        capacity_fmt = insert_number_commas(capacity)

        metrics.pct_full = self.get_pct_full(client)
        metrics.batch_size = 50
        metrics.bytes_cum = 0
        metrics.time_cum = 0
        metrics.items_cum = 0
        rate = -1

        while metrics.pct_full < self.percentage:
            self.write("Cache is %.2f%% full of %s, inserting %s items (rate: %s items/s)" %
                       (metrics.pct_full, capacity_fmt, metrics.batch_size,
                        insert_number_commas(str(int(rate)))))

            time_st = time.time()

            for _ in range(metrics.batch_size):
                if not self._runnable:
                    return

                key = generate_random_key_uuid(10)
                value = generate_random_data(100, 1000)
                client.set(key, value)

                metrics.bytes_cum += len(key) + len(value)

            duration = time.time() - time_st
            rate = metrics.batch_size / duration
            metrics.time_cum += duration
            metrics.items_cum += metrics.batch_size

            metrics.pct_full = self.get_pct_full(client)
