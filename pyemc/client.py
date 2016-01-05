from collections import OrderedDict
import re

from pyemc.socket_stream import BufferedSocketStream


class MemcacheClientParams(object):
    def __init__(self, host, port):
        self.host = host
        self.port = port

    def create_client(self):
        return MemcacheClient(
            host=self.host,
            port=self.port,
        )


class ClientError(Exception):
    pass

class DeleteFailedError(Exception):
    pass

class NotFoundError(Exception):
    pass

class NotStoredError(Exception):
    pass

class ServerError(Exception):
    pass


_exc_map = {
    'ERROR': ServerError,
    'NOT_FOUND': NotFoundError,
    'NOT_STORED': NotStoredError,
}
def create_exc(server_resp, exc_msg):
    if server_resp.startswith('CLIENT_ERROR'):
        msg = server_resp.split(' ', 1)[-1]
        return ClientError(msg)

    elif server_resp.startswith('SERVER_ERROR'):
        msg = server_resp.split(' ', 1)[-1]
        return ServerError(msg)

    exc_cls = _exc_map.get(server_resp.strip())

    if exc_cls is None:
        raise Exception("Could not create exception for: %s" % server_resp)

    return exc_cls(exc_msg)


class Item(object):
    def __init__(self, key, flags, value):
        self.key = key
        self.flags = flags
        self.value = value

    def __repr__(self):
        return '<%s key=%r, flags=%r, value=%r>' % (
            self.__class__.__name__,
            self.key,
            self.flags,
            self.value,
        )


class MemcacheClient(object):
    rx_value = re.compile('VALUE (?P<key>[^ ]*) (?P<flags>\d+) (?P<len>\d+)')
    rx_inc_value = re.compile('(?P<value>\d+)')

    def __init__(self, host, port):
        self.stream = BufferedSocketStream(host, port)

    def add(self, key, value, flags=0, exptime=0, noreply=False):
        return self._set_family('add', key, value, flags, exptime, noreply)

    def append(self, key, value, flags=0, exptime=0, noreply=False):
        return self._set_family('append', key, value, flags, exptime, noreply)

    def decr(self, key, delta=1, noreply=False):
        return self._inc_family('decr', key, delta, noreply)

    def delete(self, key, noreply=False):
        # prepare command
        command = 'delete %(key)s %(noreply)s\r\n' % {
            'key': key,
            'noreply': 'noreply' if noreply else '',
        }

        # execute command
        self.stream.write(command)

        # parse the response
        if not noreply:
            resp = self.stream.read_line()
            if not resp == 'DELETED\r\n':
                raise create_exc(resp, 'Could not delete key %r' % key)

    def flush_all(self, exptime=None, noreply=False):
        # prepare command
        # TODO no support for parameters yet
        command = 'flush_all\r\n'

        # execute command
        self.stream.write(command)

        # parse the response
        if not noreply:
            resp = self.stream.read_line()
            if not resp == 'OK\r\n':
                raise create_exc(resp, 'Could not perform flush_all')

    def get_multi(self, keys):
        # prepare command
        keys = ' '.join(keys)
        command = 'get %s\r\n' % keys

        # execute command
        self.stream.write(command)

        # parse the response
        dct = OrderedDict()
        stream_terminator = 'END\r\n'

        while True:
            line = self.stream.read_line()
            try:
                key, flags, bytelen = self.rx_value.findall(line)[0]
                flags = int(flags)
                bytelen = int(bytelen)
            except IndexError:
                # no items were returned at all
                if line == stream_terminator:
                    break

            # read value + line terminator
            data = self.stream.read_exact(bytelen + 2)

            data = data[:-2]
            item = Item(key, flags, data)
            dct[key] = item

            if self.stream.peek_contains(stream_terminator, consume=True):
                break

        return dct

    def get(self, key):
        keys = (key,)
        dct = self.get_multi(keys)

        try:
            return dct[key]
        except KeyError:
            raise NotFoundError('The item with key %r was not found' % key)

    def get_stats(self):
        dct = OrderedDict()

        # prepare command
        command = 'stats\r\n'

        # execute command
        self.stream.write(command)

        # read response line by line
        stream_terminator = 'END\r\n'

        line = self.stream.read_line()
        while line != stream_terminator:
            kw, key, value = line.split(' ', 2)
            dct[key] = value.strip()

            line = self.stream.read_line()

        return dct

    def incr(self, key, delta=1, noreply=False):
        return self._inc_family('incr', key, delta, noreply)

    def _inc_family(self, instr, key, delta=1, noreply=False):
        # prepare command
        command = '%(instr)s %(key)s %(delta)d %(noreply)s\r\n' % {
            'instr': instr,
            'key': key,
            'delta': int(delta),
            'noreply': 'noreply' if noreply else '',
        }

        # execute command
        self.stream.write(command)

        # read the response
        if not noreply:
            resp = self.stream.read_line()
            try:
                num = self.rx_inc_value.findall(resp)[0]
                return num
            except IndexError:
                raise create_exc(resp, 'Could not %s key %r' % (instr, key))

    def quit(self):
        '''Tells the server to drop the connection.'''

        # prepare command
        command = 'quit\r\n'

        # execute command
        self.stream.write(command)

    def prepend(self, key, value, flags=0, exptime=0, noreply=False):
        return self._set_family('prepend', key, value, flags, exptime, noreply)

    def replace(self, key, value, flags=0, exptime=0, noreply=False):
        return self._set_family('replace', key, value, flags, exptime, noreply)

    def set(self, key, value, flags=0, exptime=0, noreply=False):
        return self._set_family('set', key, value, flags, exptime, noreply)

    def _set_family(self, instr, key, value, flags=0, exptime=0, noreply=False):
        # prepare command
        header = '%(instr)s %(key)s %(flags)d %(exptime)d %(bytelen)d %(noreply)s\r\n' % {
            'instr': instr,
            'key': key,
            'flags': flags,
            'exptime': exptime,
            'bytelen': len(value),
            'noreply': 'noreply' if noreply else '',
        }
        command = header + value + '\r\n'

        # execute command
        self.stream.write(command)

        # check for success
        if not noreply:
            resp = self.stream.read_line()
            if not resp == 'STORED\r\n':
                raise create_exc(resp, 'Could not %s key %r to %r...' %
                                 (instr, key, value[:10]))

    def send_malformed_cmd(self):
        '''Sends an invalid command - causes the server to drop the
        connection'''

        self.stream.write('set 0 1\r\n')
        buf = self.stream.read(4096)
        return buf.strip()

    def touch(self, key, exptime=0, noreply=False):
        # prepare command
        command = 'touch %(key)s %(exptime)d %(noreply)s\r\n' % {
            'key': key,
            'exptime': exptime,
            'noreply': 'noreply' if noreply else '',
        }

        # execute command
        self.stream.write(command)

        # check for success
        if not noreply:
            resp = self.stream.read_line()
            if not resp == 'TOUCHED\r\n':
                raise create_exc(resp, 'Could not touch key %r' % key)

    def version(self):
        # prepare command
        command = 'version\r\n'

        # execute command
        self.stream.write(command)

        # check for success
        resp = self.stream.read_line()
        kw, version = resp.split(' ', 1)
        return version.strip()
