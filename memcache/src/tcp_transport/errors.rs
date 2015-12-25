#[derive(Debug, PartialEq)]
pub enum TcpTransportError {
    CommandParseError,
    InvalidCmd,
    LineReadError,
    NumberParseError,
    StreamReadError,
    Utf8Error,
    WordReadError,
}
