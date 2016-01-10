# Current (eliding Result in return value)

    read_bytes_exact(len: u64) -> Vec<u8>

    read_word_in_line() -> Vec<u8>, end_of_line: bool

    read_line_as_words() -> Vec<Vec<u8>>


# Desired


## Parameters

    delimiter: String - What separates words. Space in our case.
    sloppy_delimiter: bool - Allows multiple spaces between tokens

## Modes

    set_line_mode() -> ()

Allows reading words until the end of the line, then starts returning errors.

    set_byte_mode() -> ()

Disallows reading words, only allows reading bytes.

## Reading bytes

    read_bytes_exact(len: u64) -> Vec<u8>

For reading payloads (length prefixed).

    read_line_terminator() -> Result<()>

Curried version of read_bytes_exact.

## Reading words

    read_word_as::<T>() -> Result<T>

For reading keywords, keys, int fields.

## Lookahead

    next_word_is_type::<T>() -> Result<T>

Match against known type of value, return the value if the type matches.

    next_word_is_value::<T>(value: T) -> bool

Match against known value (for keywords like noreply).
