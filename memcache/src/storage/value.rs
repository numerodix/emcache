#![macro_use]


#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Value {
    item: Vec<u8>,
}

impl Value {
    pub fn new(item: Vec<u8>) -> Value {
        Value { item: item }
    }

    pub fn len(&self) -> usize {
        self.item.len()
    }
}

// value!(1, 2, 3) => Value { item: Vec<u8> = [1, 2, 3] }
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
