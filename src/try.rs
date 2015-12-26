#[derive(Debug, Clone)]
struct Value {
    item: Vec<u8>,

    atime_entry: Option<Box<AccessTimeEntry>>,
}


#[derive(Debug, Clone)]
struct AccessTimeEntry {
    atime: f64,

    value_entry: Option<Box<Value>>,
}


pub fn may() {
    let mut value = Value {
        item: vec![1],
        atime_entry: None,
    };

    let mut atime = AccessTimeEntry {
        atime: 0.0,
        value_entry: None,
    };

    value.atime_entry = Some(Box::new(atime.clone()));
    atime.value_entry = Some(Box::new(value.clone()));

    println!("value: {:?}", value);
    println!("atime: {:?}", atime);
}
