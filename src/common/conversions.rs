use std::str;


pub fn string_to_str<'a>(st: &'a String) -> &'a str {
    // Not really unsafe because the String is already valid so..
    unsafe { str::from_utf8_unchecked(st.as_bytes()) }
}
