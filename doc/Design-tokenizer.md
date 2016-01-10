# Current (eliding Result in return value)

    read_bytes_exact(len: u64) -> Vec<u8>

    read_word_in_line() -> Vec<u8>, end_of_line: bool

    read_line_as_words() -> Vec<Vec<u8>>


# Desired


## Modes

    set_line_mode() -> ()

allows reading words until the end of the line, then fails

    set_byte_mode() -> ()

disallows reading words, only allows reading bytes

## Reading bytes

    read_bytes_exact(len: u64) -> Vec<u8>

for payloads (length prefixed)

    read_line_terminator() -> Result<()>

curried version of read_bytes_exact

## Reading words

    read_word_as::<T> -> Result<T>

for reading keywords, keys, int fields

## Lookahead

    next_word_is_type::<T> -> Result<T>

match against known type of value, return value if type matches

    next_word_is_value::<T>(value: T) -> bool

match against known value (for keywords like noreply)
