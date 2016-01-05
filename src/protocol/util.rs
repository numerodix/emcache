use platform::time::time_now;
use storage::CacheError;

use super::cmd::Resp;


pub fn bytes_to_u64(bytes: &Vec<u8>) -> Option<u64> {
    match String::from_utf8(bytes.clone()) {
        Ok(st) => {
            match st.parse::<u64>() {
                Ok(num) => Some(num),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

pub fn u64_to_bytes(num: &u64) -> Vec<u8> {
    let arr = num.to_string().into_bytes();

    let mut bytes = vec![];
    bytes.extend(arr);
    bytes
}


pub fn convert_exptime(exptime: u32) -> Option<f64> {
    // If exptime is greater than zero it means it's set, otherwise unset
    if exptime > 0 {
        let tm;

        // Is it an interval greater than 30 days? Then it's a timestamp
        if exptime > 60 * 60 * 24 * 30 {
            tm = exptime as f64;

        } else {
            // Otherwise it's relative from now
            tm = time_now() + exptime as f64;
        }

        return Some(tm);
    }

    None
}

pub fn from_cache_err(err: &CacheError) -> Resp {
    match *err {
        CacheError::KeyTooLong => {
            Resp::ClientError("bad command line format".to_string())
        }
        CacheError::ValueTooLong => {
            Resp::ServerError("object too large for cache".to_string())
        }
        _ => Resp::Error,
    }
}


#[test]
fn test_bytes_to_u64() {
    // whitespace
    assert_eq!(None, bytes_to_u64(&vec![b' ', b'2']));

    // alpha
    assert_eq!(None, bytes_to_u64(&vec![b'0', b'x', b'2']));

    // negative
    assert_eq!(None, bytes_to_u64(&vec![b'-', b'2']));

    // too long for u64
    assert_eq!(None, bytes_to_u64(&vec![b'1'; 255]));

    // ok
    assert_eq!(12, bytes_to_u64(&vec![b'1', b'2']).unwrap());
}

#[test]
fn test_u64_to_bytes() {
    // any u64 is representable as bytes so there are no boundary conditions to
    // check
    assert_eq!(vec![b'1', b'2'], u64_to_bytes(&12));
}
