#[derive(Debug, PartialEq)]
pub enum TcpTransportError {
    CommandParseError,
    InvalidCmd,
    LineReadError,
    NumberParseError,
    StreamReadError,
    StreamWriteError,
    Utf8Error,
    WordReadError,
}
