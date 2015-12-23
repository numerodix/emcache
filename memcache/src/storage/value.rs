use super::utils::time_now_utc;


#[derive(Debug, Clone)]
pub struct Value {
    pub item: Vec<u8>,
    pub atime: f64,
}

impl PartialEq for Value {
    // Overload eq to make sure we don't compare atime's
    fn eq(&self, other: &Value) -> bool {
        self.item == other.item
    }
}

impl Value {
    pub fn new(item: Vec<u8>) -> Value {
        Value {
            item: item,
            atime: -1.0,
        }
    }

    pub fn len(&self) -> usize {
        self.item.len()
    }

    pub fn touch(&mut self) {
        self.atime = time_now_utc();
    }
}
