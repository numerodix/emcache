use std::str;


pub fn string_to_str<'a>(st: &'a String) -> &'a str {
    // Not really unsafe because the String is already valid so..
    unsafe { str::from_utf8_unchecked(st.as_bytes()) }
}


#[cfg(test)]
mod tests {
    use super::string_to_str;


    #[test]
    fn test_string_to_str() {
        let st = "abc".to_string();
        assert_eq!(&st, string_to_str(&st));
    }
}
