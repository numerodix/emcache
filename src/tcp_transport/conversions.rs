use std::str::FromStr;

use super::errors::TcpTransportError;
use super::typedefs::TcpTransportResult;


pub fn as_string(bytes: Vec<u8>) -> TcpTransportResult<String> {
    match String::from_utf8(bytes) {
        Ok(st) => Ok(st),
        Err(_) => Err(TcpTransportError::Utf8Error),
    }
}

pub fn as_number<N: FromStr>(bytes: Vec<u8>) -> TcpTransportResult<N> {
    let string = try!(as_string(bytes));
    match string.parse::<N>() {
        Ok(num) => Ok(num),
        Err(_) => Err(TcpTransportError::NumberParseError),
    }
}



#[cfg(test)]
mod tests {
    use tcp_transport::TcpTransportError;

    use super::as_number;
    use super::as_string;


    #[test]
    fn test_as_string() {
        // bytestring is utf8
        let st = as_string(vec![b'a', b'b']).unwrap();
        assert_eq!(st, "ab".to_string());

        // bytestring is not utf8
        let err = as_string(vec![b'a', 254, b'b']).unwrap_err();
        assert_eq!(err, TcpTransportError::Utf8Error);
    }

    #[test]
    fn test_as_number() {
        // bytestring is a number
        let num = as_number::<u64>(vec![b'1', b'2']).unwrap();
        assert_eq!(num, 12);

        // bytestring is not a number
        let err = as_number::<u64>(vec![b' ', b'1', b'2']).unwrap_err();
        assert_eq!(err, TcpTransportError::NumberParseError);
    }
}
