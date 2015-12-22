#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Value {
    pub item: Vec<u8>,
}

impl Value {
    pub fn new(item: Vec<u8>) -> Value {
        Value { item: item }
    }

    pub fn len(&self) -> usize {
        self.item.len()
    }
}
