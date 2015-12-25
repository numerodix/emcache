use super::errors::TcpTransportError;


pub type TcpTransportResult<T> = Result<T, TcpTransportError>;
