use std::str::FromStr;

use super::errors::TcpTransportError;
use super::typedefs::TcpTransportResult;


pub fn as_string(bytes: &[u8]) -> TcpTransportResult<String> {
    // TODO fix bogus conversion without checks
    let st = String::from_utf8_lossy(bytes);
    return Ok(st.to_string());
}

pub fn as_number<N: FromStr>(bytes: &[u8]) -> TcpTransportResult<N> {
    let string = try!(as_string(bytes));
    match string.parse::<N>() {
        Ok(num) => Ok(num),
        Err(_) => Err(TcpTransportError::NumberParseError),
    }
}
