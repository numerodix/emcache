use std::str;
use std::str::FromStr;

use super::errors::TcpTransportError;
use super::typedefs::TcpTransportResult;


pub fn as_string(bytes: &[u8]) -> TcpTransportResult<String> {
    match str::from_utf8(bytes) {
        Ok(st) => Ok(st.to_string()),
        Err(_) => Err(TcpTransportError::Utf8Error),
    }
}

pub fn as_number<N: FromStr>(bytes: &[u8]) -> TcpTransportResult<N> {
    let string = try!(as_string(bytes));
    match string.parse::<N>() {
        Ok(num) => Ok(num),
        Err(_) => Err(TcpTransportError::NumberParseError),
    }
}
