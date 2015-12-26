#!/usr/bin/env python

import random
import re
import socket


def generate_random_data(length_from, length_to=None):
    length = length_from
    if length_to is not None:
        length = random.randint(length_from, length_to)

    with open('/dev/urandom', 'rb') as f:
        return f.read(length)

def generate_random_key(length):
    data = ''
    while len(data) < length:
        bytes = generate_random_data(length)
        # filter out non printable chars
        bytes = [b for b in bytes
                 if 65 <= ord(b) <= 90 or 97 <= ord(b) <= 122]
        bytes = ''.join(bytes)
        data += bytes
    data = data[:length]
    return data


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
    from optparse import OptionParser
    parser = OptionParser()
    parser.add_option('-n', '', action='store', dest='host',
                      help='Host to connect to')
    parser.add_option('-p', '', action='store', type='int', dest='port',
                      help='Port to connect to')
    (options, args) = parser.parse_args()


    host = options.host is not None and options.host or '127.0.0.1'
    port = options.port is not None and int(options.port) or 11211


    client = Client(host, port)
    client.print_stats()

    key = generate_random_key(4)
    val = generate_random_data(5, 8)

    print("Setting key:   %r -> %r" % (key, val))
    client.set(key, val)

    val2 = client.get(key)
    print("Retrieved key: %r -> %r" % (key, val2))

    assert val == val2


    #value = client.get('y')
    #print("Retrieved 'y' -> '%s'" % value)

    #resp = client.send_malformed_cmd()
    #print("Sent malformed command, got '%s'" % resp)
