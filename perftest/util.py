import os
import random
import uuid


def generate_random_data(length_from, length_to=None):
    length = length_from
    if length_to is not None:
        length = random.randint(length_from, length_to)

    return os.urandom(length)

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

def generate_random_key_uuid(length_from, length_to=None):
    length = length_from
    if length_to is not None:
        length = random.randint(length_from, length_to)

    # XXX length will never be >32
    return uuid.uuid4().hex[:length]

def insert_number_commas(number):
    chunks = []

    num_iterations = (len(number) / 3) + 1
    idx = -3

    for i in range(1, num_iterations + 1):
        chunk = number[idx:]
        number = number[:idx]
        chunks.append(chunk)
        if i != num_iterations:
            chunks.append(',')

    chunks.reverse()
    number = ''.join(chunks)
    if number.startswith(','):
        number = number[1:]

    return number


if __name__ == '__main__':
    import time

    def gen(num, num_chars, gen_func, type_name):
        time_start = time.time()

        for i in xrange(num):
            gen_func(num_chars)

        time_stop = time.time()

        duration = time_stop - time_start
        rate_strings = float(num) / duration
        rate_bytes = rate_strings * num_chars
        print("Generated %s %s-char %s in %.2fs (rate: %s strings/s - %s bytes/s)" %
              (insert_number_commas(str(num)),
               insert_number_commas(str(num_chars)),
               type_name,
               duration,
               insert_number_commas(str(int(rate_strings))),
               insert_number_commas(str(int(rate_bytes)))
              ))

    gen(100000, 100, generate_random_data, 'byte strings')
    gen(10000, 100, generate_random_key, 'alphanum strings')
    gen(10000, 100, generate_random_uuid, 'alphanum strings')
