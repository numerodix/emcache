from contextlib import contextmanager
import time
import traceback
import sys


def write(msg):
    msg = msg + '\n' if not msg.endswith('\n') else msg
    sys.stdout.write(msg)
    sys.stdout.flush()


class TestFailedError(Exception):
    pass


class TestRunner(object):
    def __init__(self, client_params):
        self.client_params = client_params

    def execute_all(self, test_cases_classes):
        test_id = -1

        failed = []
        passed = []

        for test_case_cls in test_cases_classes:
            atts = dir(test_case_cls)
            atts = [att for att in atts if att.startswith('test_')]

            for att in atts:
                unbound_method = getattr(test_case_cls, att)
                if not callable(unbound_method):
                    continue

                test_id += 1
                test_case = test_case_cls(id=test_id)

                bound_method = getattr(test_case, att)
                rv = self.execute_one(bound_method)

                if rv:
                    passed.append(att)
                else:
                    failed.append(att)

        self.write("%s test run: %s passed, %s failed" %
                   (len(failed + passed), len(passed), len(failed)))

        if failed:
            return False

        return True

    def execute_one(self, method):
        test_case = method.im_self
        test_name = '%s.%s' % (test_case.__class__.__name__, method.__name__)
        exc = None

        self.write("Running test %s" % test_name)
        try:
            test_case.set_up(self.client_params)
        except Exception as e:
            traceback.print_exc()
            self.write("Test %s set_up FAILED: %r" % e)
            return

        time_start = time.time()
        try:
            method()
        except Exception as e:
            exc = e
            traceback.print_exc()

        time_stop = time.time()
        dur = time_stop - time_start

        if exc is None:
            self.write("SUCCEEDED: %s in %.4fs" % (test_name, dur))
            return True
        else:
            self.write("FAILED: %s in %.4fs: %r" % (test_name, dur, exc))
            return False

    def write(self, msg):
        msg = '[runner] %s' % msg
        write(msg)


class TestCase(object):
    def __init__(self, id):
        self.id = id

    def set_up(self, client_params):
        '''Subclasses should implement both set_up and get_client so the runner
        can get access to the client.'''
        self.client = client_params.create_client()

    def get_client(self):
        return self.client

    @contextmanager
    def assert_raises(self, exc_class):
        raised = False
        try:
            yield

        except exc_class:
            raised = True

        if not raised:
            raise AssertionError("%s not raised" % exc_class.__name__)

    def write(self, msg):
        msg = '[test%s] %s' % (self.id, msg)
        write(msg)
