#[derive(Debug, Clone)]
struct Value {
    item: Vec<u8>,

    atime_entry: Option<*mut AccessTimeEntry>,
}


#[derive(Debug, Clone)]
struct AccessTimeEntry {
    atime: f64,

    value_entry: Option<*mut Value>,
}


pub fn must() {
    let mut atime = AccessTimeEntry {
        atime: 1.0,
        value_entry: None,
    };

    let mut value = Value {
        item: vec![1],
        atime_entry: None,
    };

    value.atime_entry = Some(&mut atime);
    atime.value_entry = Some(&mut value);

    println!("value: {:?}", value);
    println!("atime: {:?}", atime);

    unsafe {
        let ref mut val = *atime.value_entry.unwrap();
        val.item = vec![2];
    }

    unsafe {
        let ref mut at = *value.atime_entry.unwrap();
        at.atime = 2.0;
    }

    println!("value: {:?}", value);
    println!("atime: {:?}", atime);
}
