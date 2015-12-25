#[derive(Debug, PartialEq)]
pub enum TcpTransportError {
    CommandParseError,
    InvalidCmd,
    LineReadError,
    SocketReadError,
    Utf8Error,
    WordReadError,
}
