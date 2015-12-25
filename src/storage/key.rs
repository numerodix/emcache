#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Key {
    pub item: Vec<u8>,
}

impl Key {
    pub fn new(item: Vec<u8>) -> Key {
        Key { item: item }
    }

    pub fn len(&self) -> usize {
        self.item.len()
    }
}
