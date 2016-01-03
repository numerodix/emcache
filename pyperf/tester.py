#!/usr/bin/env python

import re
import socket
import sys
import time

from pyperf.client import MemcacheClient
from pyperf.client import MemcacheClientParams
from pyperf.task_filler import CacheFillerTask


if __name__ == '__main__':
    from optparse import OptionParser
    parser = OptionParser()
    parser.add_option('-n', '', action='store', dest='host',
                      help='Host to connect to')
    parser.add_option('-p', '', action='store', type='int', dest='port',
                      help='Port to connect to')
    parser.add_option('', '--fill', action='store', type='float', dest='fill_cache',
                      help='Fill the cache to a given percentage full')
    parser.add_option('', '--stress', action='store_true', dest='stress_test',
                      help='Perform a stress test')
    parser.add_option('-w', '--workers', action='store', type='int', dest='worker_count',
                      help='Use these many worker threads')
    (options, args) = parser.parse_args()


    host = options.host is not None and options.host or '127.0.0.1'
    port = options.port is not None and int(options.port) or 11311


    cli_params = MemcacheClientParams(host, port)

    if options.fill_cache:
        pct = float(options.fill_cache)
        filler = CacheFillerTask(
            client_params=cli_params,
            percentage=pct,
            jobs=options.worker_count,
        )
        filler.launch()

    elif options.stress_test:
        client = cli_params.create_client()
        # establish connection
        client.get('invalid')

        # measure set rate
        start_time = time.time()
        num = 10000  # should take about 2secs
        for _ in range(num):
            client.set('x', 'abc')
        end_time = time.time()
        set_interval = end_time - start_time
        set_rate = float(num) / (set_interval) * 2

        # measure set rate
        start_time = time.time()
        num = 10000  # should take about 2secs
        for _ in range(num):
            client.get('x')
        end_time = time.time()
        get_interval = end_time - start_time
        get_rate = float(num) / (get_interval) * 2

        # print stats afterwards
        client.print_stats()

        # display results
        print("Made %d constant key set requests in %.2f seconds = %.2f requests/sec" %
              (num, set_interval, set_rate))
        print("Made %d constant key get requests in %.2f seconds = %.2f requests/sec" %
              (num, get_interval, get_rate))

    else:
        from pyperf.util import generate_random_key
        from pyperf.util import generate_random_data

        client = cli_params.create_client()



        key = generate_random_key(4)
        val = generate_random_data(5, 8)

        print("Setting small key:   %r -> %r" % (key, val))
        client.set(key, val)

        val2 = client.get(key)
        print("Retrieved small key: %r -> %r" % (key, val2))

        assert val == val2



        key = generate_random_key(15)
        val = generate_random_data(1 << 19)  # .5mb

        print("Setting large key:   %r -> %r..." % (key, val[:20]))
        client.set(key, val)

        val2 = client.get(key)
        print("Retrieved large key: %r -> %r..." % (key, val2[:20]))

        assert val == val2



        value = client.get('y')
        print("Retrieved 'y' -> '%s'" % value)

        client.print_stats()

        resp = client.send_malformed_cmd()
        print("Sent malformed command, got '%s'" % resp)
