#[derive(Debug, PartialEq)]
pub enum TcpTransportError {
    CommandParseError,
    InvalidCmd,
    LineReadError,
    NumberParseError,
    SocketReadError,
    Utf8Error,
    WordReadError,
}
