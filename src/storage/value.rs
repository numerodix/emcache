use platform::time::time_now;


#[derive(Debug, Clone)]
pub struct Value {
    pub item: Vec<u8>,
    pub atime: f64, // last access time (unixtime)
    pub exptime: f64, // expiry time (unixtime), <0 for unset
}

impl PartialEq for Value {
    // Overload eq to make sure we only compare the payloads
    fn eq(&self, other: &Value) -> bool {
        self.item == other.item
    }
}

impl Value {
    pub fn new(item: Vec<u8>) -> Value {
        Value {
            item: item,
            atime: -1.0,
            exptime: -1.0,
        }
    }

    pub fn len(&self) -> usize {
        self.item.len()
    }

    pub fn touch(&mut self) {
        self.atime = time_now();
    }

    pub fn set_exptime(&mut self, exptime: f64) {
        self.exptime = exptime;
    }
}
