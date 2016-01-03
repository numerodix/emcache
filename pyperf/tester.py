#!/usr/bin/env python

from __future__ import print_function

import re
import socket
import sys
import time

from pyperf.abstractions.test_api import TestRunner
from pyperf.client import ItemNotFoundError
from pyperf.client import MemcacheClient
from pyperf.client import MemcacheClientParams
from pyperf.task_filler import CacheFillerTask
from pyperf.test_integration import TestApi
from pyperf.test_stress import TestStress


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
        runner = TestRunner(cli_params)

        test_cases = [
            TestStress,
        ]

        rv = runner.execute_all(test_cases)
        sys.exit(not rv)

    else:
        runner = TestRunner(cli_params)

        test_cases = [
            TestApi,
        ]

        rv = runner.execute_all(test_cases)
        sys.exit(not rv)
