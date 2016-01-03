import signal
import threading
import sys


def write(msg):
    msg = msg + '\n' if not msg.endswith('\n') else msg
    sys.stderr.write(msg)
    sys.stderr.flush()


class TaskletMetrics(object):
    '''This class allows a Tasklet to store any state that a Task may need to
    have after the execution of its Tasklets.'''

    def __init__(self):
        pass


class Tasklet(object):
    '''A Tasklet represents a unit of work run in a separate thread.'''

    def __init__(self, id, client_params):
        self.id = id
        self.client_params = client_params

        self._metrics = TaskletMetrics()
        self._runnable = True

    def launch(self):
        client = self.client_params.create_client()
        self.run(client, self._metrics)

    def run(self, client):
        raise NotImplementedError

    def write(self, msg):
        msg = '[thread%s] %s' % (self.id, msg)
        write(msg)


class TaskState(object):
    '''This class represents any state that a task needs to keep between task
    phases.'''

    def __init__(self):
        pass


class Task(object):
    '''Represents a task run against the server, which internally delegates its
    work to Tasklets run in threads.
    
    Users need to implement a Task to do any setup/teardown of the task and
    define the creation of Tasklets.'''

    def __init__(self, client_params):
        self.client_params = client_params

    def create_tasklets(self):
        raise NotImplementedError

    def pre_tasklets(self):
        raise NotImplementedError

    def run_tasklets(self, tasklets):
        metrics_list = []
        threads = []

        def handle_signal(signal, frame):
            write("Got interrupt, stopping all tasklets")
            for tasklet in tasklets:
                tasklet._runnable = False

        signal.signal(signal.SIGINT, handle_signal)

        for tasklet in tasklets:
            thread = threading.Thread(target=tasklet.launch)
            thread.daemon = True
            metrics_list.append(tasklet._metrics)
            threads.append(thread)
            thread.start()

        while threads:
            for thread in threads:
                thread.join(.05)
                if not thread.isAlive():
                    threads.remove(thread)

        return metrics_list

    def post_tasklets(self):
        raise NotImplementedError

    def launch(self):
        client = self.client_params.create_client()
        state = TaskState()

        tasklets = self.create_tasklets(state)
        self.pre_tasklets(client, state)
        metrics_list = self.run_tasklets(tasklets)
        self.post_tasklets(client, state, metrics_list)

    def write(self, msg):
        write(msg)
