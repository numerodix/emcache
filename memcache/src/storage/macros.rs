#![macro_use]


// key!(1, 2, 3) => Key { item: Vec<u8> = [1, 2, 3] }
#[macro_export]
macro_rules! key {
    ( $( $x:expr ),* ) => {
        {
            let mut vec = Vec::new();
            $(
                vec.push($x);
            )*
            Key::new(vec)
        }
    };
}

// value!(1, 2, 3) => Value { item: Vec<u8> = [1, 2, 3] }
#[macro_export]
macro_rules! value {
    ( $( $x:expr ),* ) => {
        {
            let mut vec = Vec::new();
            $(
                vec.push($x);
            )*
            Value::new(vec)
        }
    };
}
