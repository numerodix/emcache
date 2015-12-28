#!/usr/bin/env python

import re
import socket
import time
import sys

from perftest.client import MemcacheClient
from perftest.client import MemcacheClientParams
from perftest.loadgen import LoadGenerator


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
    (options, args) = parser.parse_args()


    host = options.host is not None and options.host or '127.0.0.1'
    port = options.port is not None and int(options.port) or 11311


    cli_params = MemcacheClientParams(host, port)

    if options.fill_cache:
        loadgen = LoadGenerator(cli_params)
        pct = float(options.fill_cache)
        loadgen.fill_to_pct(pct)

    elif options.stress_test:
        client = cli_params.create_client()
        # establish connection
        client.get('invalid')

        # measure transaction rate
        start_time = time.time()
        num = 10000  # should take about 2secs
        for _ in range(num):
            client.set('x', '1')
            val2 = client.get('x')
        end_time = time.time()

        # print stats afterwards
        client.print_stats()

        # display results
        interval = end_time - start_time
        rate = float(num) / (interval) * 2
        print("Made %d set+get requests in %.2f seconds = %.2f requests/sec" %
              (num, interval, rate))

    else:
        from perftest.util import generate_random_key
        from perftest.util import generate_random_data

        client = cli_params.create_client()

        key = generate_random_key(4)
        val = generate_random_data(5, 8)

        print("Setting key:   %r -> %r" % (key, val))
        client.set(key, val)

        val2 = client.get(key)
        print("Retrieved key: %r -> %r" % (key, val2))

        assert val == val2


        value = client.get('y')
        print("Retrieved 'y' -> '%s'" % value)

        client.print_stats()

        resp = client.send_malformed_cmd()
        print("Sent malformed command, got '%s'" % resp)
