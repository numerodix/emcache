use std::mem;


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

    pub fn mem_size(&self) -> usize {
        mem::size_of::<Self>() + self.item.len()
    }
}
