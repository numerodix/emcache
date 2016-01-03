import socket


def connected(func):
    def new_func(self, *args, **kwargs):
        if self.sock is None:
            self.connect()
        return func(self, *args, **kwargs)
    return new_func


class BufferedSocketStream(object):
    def __init__(self, host, port, line_terminator='\r\n'):
        self.host = host
        self.port = port
        self.line_terminator = line_terminator

        self.sock = None
        self.std_read_size = 4096
        self.read_ahead = ''

    def connect(self):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((self.host, self.port))
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)

    @connected
    def read(self, count):
        '''Attempts to satisfy the read from the read ahead buffer. Failing
        that it will recv() once from the socket and return whatever was
        there.'''

        if self.read_ahead:
            buf = self.read_ahead
            self.read_ahead = ''

            if len(buf) < count:
                left_to_read = count - len(buf)
                chunk = self.sock.recv(left_to_read)
                buf += chunk

            return buf

        buf = self.sock.recv(count)
        return buf

    @connected
    def read_exact(self, count):
        '''Reads the exact number of bytes from the socket, in as many recv()
        calls as necessary.'''

        buf = self.read_ahead

        while len(buf) < count:
            left_to_read = count - len(buf)
            chunk = self.sock.recv(left_to_read)
            buf += chunk

        if len(buf) >= count:
            self.read_ahead = buf[count:]
            buf = buf[:count]

        return buf

    @connected
    def read_line(self):
        '''Reads one line from the socket, in as many recv() calls as
        necessary. Stores any data left over in the read ahead buffer.'''

        line = self.read_ahead

        while not self.line_terminator in line:
            chunk = self.sock.recv(self.std_read_size)
            line += chunk

        idx = line.index(self.line_terminator)
        if idx + 2 <= len(line):
            self.read_ahead = line[idx + 2:]
            line = line[:idx + 2]

        return line

    def peek_contains(self, token, consume=False):
        '''Peeks at the stream for an expected value without consuming the
        stream. Tries to satisfy the peek from the read ahead buffer. Failing
        that, it calls read_exact() to read the number of bytes to match the
        length of the token.'''

        if len(self.read_ahead) < len(token):
            left_to_read = len(token) - len(self.read_ahead)
            self.read_ahead += self.read_exact(left_to_read)

        if self.read_ahead[:len(token)] == token:
            if consume:
                self.read_ahead = self.read_ahead[len(token):]

            return True

        return False

    @connected
    def write(self, buf):
        '''Writes the complete buffer to the socket.'''

        self.sock.sendall(buf)
