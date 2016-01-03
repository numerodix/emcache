from collections import OrderedDict
import re
import socket


class MemcacheClientParams(object):
    def __init__(self, host, port):
        self.host = host
        self.port = port

    def create_client(self):
        return MemcacheClient(
            host=self.host,
            port=self.port,
        )


def connected(func):
    def new_func(self, *args, **kwargs):
        if self.sock is None:
            self.connect()
        return func(self, *args, **kwargs)
    return new_func


class MemcacheClient(object):
    rx_get_resp = re.compile('VALUE (?P<key>[^ ]*) \d+ (?P<len>\d+)')

    def __init__(self, host, port):
        self.host = host
        self.port = port

        self.sock = None

    def connect(self):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((self.host, self.port))
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)

    @connected
    def get_stats(self):
        self.sock.send('stats\r\n')
        buf = self.sock.recv(4096)
        # remove the terminator
        end_pos = buf.find('END')
        buf = buf[:end_pos]

        dct = OrderedDict()
        lines = buf.splitlines()
        for line in lines:
            kw, key, value = line.split(' ', 2)
            dct[key] = value

        return dct

    def print_stats(self):
        dct = self.get_stats()
        for (key, value) in dct.items():
            print('%s: %s' % (key, value))

    @connected
    def set(self, key, value):
        # execute set
        self.sock.send(
            'set %s 0 0 %s \r\n' % (key, len(value))
            + value + '\r\n')
        buf = self.sock.recv(4096)

        # parse response
        if not buf == 'STORED\r\n':
            raise Exception("Failed to store %s: %s" % (key, buf))

    @connected
    def get(self, key):
        # execute get
        self.sock.send('get %s\r\n' % key)
        buf = self.sock.recv(4096)

        # parse response
        header, rest = buf.split('\r\n', 1)
        try:
            _, bytelen = self.rx_get_resp.findall(header)[0]
            bytelen = int(bytelen)

            value = rest[:bytelen]

            while len(value) < bytelen:
                buf = self.sock.recv(4096)
                value += buf
                value = value[:bytelen]

            return value
        except IndexError:
            return buf.strip()

    def send_malformed_cmd(self):
        self.sock.send('set 0 0\r\n')
        buf = self.sock.recv(4096)
        return buf.strip()
