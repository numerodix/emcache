import random


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
    return ''.join(chunks)
