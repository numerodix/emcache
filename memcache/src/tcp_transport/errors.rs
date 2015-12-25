#[derive(Debug, PartialEq)]
pub enum TcpTransportError {
    CommandParseError,
    InvalidCmd,
    LineReadError,
    SocketReadError,
    WordReadError,
}
