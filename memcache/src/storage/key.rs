#![macro_use]


#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Key {
    item: Vec<u8>,
}

impl Key {
    pub fn new(item: Vec<u8>) -> Key {
        Key { item: item }
    }

    pub fn len(&self) -> usize {
        self.item.len()
    }
}

// key!(1, 2, 3) => Key { item: Vec<u8> = [1, 2, 3] }
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
