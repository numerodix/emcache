#[derive(Debug, PartialEq)]
pub enum TcpTransportError {
    InvalidCmd,
    LineReadError,
    SocketReadError,
}
