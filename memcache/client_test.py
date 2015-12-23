#!/usr/bin/env python

import re
import socket


def connected(func):
    def new_func(self, *args, **kwargs):
        if self.sock is None:
            self.connect()
        return func(self, *args, **kwargs)
    return new_func


class Client(object):
    rx_get_resp = re.compile('VALUE (?P<key>[^ ]*) \d+ (?P<len>\d+)')

    def __init__(self, host, port):
        self.host = host
        self.port = port

        self.sock = None

    def connect(self):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((self.host, self.port))

    @connected
    def print_stats(self):
        self.sock.send('stats\r\n')
        buf = self.sock.recv(4096)
        print(buf)

    @connected
    def set(self, key, value):
        # execute set
        self.sock.send(
            'set %s 0 0 %s\r\n' % (key, len(value))
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
            return value
        except IndexError:
            return buf.strip()

    def send_malformed_cmd(self):
        self.sock.send('set 0 0\r\n')
        buf = self.sock.recv(4096)
        return buf.strip()


if __name__ == '__main__':
    client = Client('127.0.0.1', 11211)
    client.print_stats()

    print("Setting 'x' to 'abc'")
    client.set('x', 'abc')

    value = client.get('x')
    print("Retrieved 'x' -> '%s'" % value)

    value = client.get('y')
    print("Retrieved 'y' -> '%s'" % value)

    resp = client.send_malformed_cmd()
    print("Sent malformed command, got '%s'" % resp)
