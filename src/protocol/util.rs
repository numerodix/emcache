use platform::time::time_now;
use storage::CacheError;

use super::cmd::Resp;


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
