use platform::time::time_now;


#[derive(Debug, Clone)]
pub struct Value {
    // Settable/gettable
    item: Vec<u8>,
    flags: u16, // chosen by the client
    exptime: f64, // expiry time (unixtime), <0 for unset

    // Managed internally
    atime: f64, // last access time (unixtime)
    cas_id: u64, // Incremented every time the value is changed
}

impl PartialEq for Value {
    // Overload eq to make sure we only compare the fields that the client
    // stores explicitly
    fn eq(&self, other: &Value) -> bool {
        self.item == other.item && self.flags == other.flags
    }
}

impl Value {
    pub fn new(item: Vec<u8>) -> Value {
        Value {
            item: item,
            flags: 0,
            atime: -1.0,
            exptime: -1.0,
            cas_id: 0,
        }
    }

    pub fn empty() -> Value {
        Value {
            item: vec![],
            flags: 0,
            atime: -1.0,
            exptime: -1.0,
            cas_id: 0,
        }
    }


    pub fn get_item_mut(&mut self) -> &mut Vec<u8> {
        self.bump_cas_id();
        &mut self.item
    }

    pub fn get_item(&self) -> &Vec<u8> {
        &self.item
    }

    pub fn set_item(&mut self, item: Vec<u8>) -> &mut Self {
        self.bump_cas_id();
        self.item = item;
        self
    }

    pub fn get_flags(&self) -> &u16 {
        &self.flags
    }

    pub fn set_flags(&mut self, flags: u16) -> &mut Self {
        self.bump_cas_id();
        self.flags = flags;
        self
    }

    pub fn get_exptime(&self) -> &f64 {
        &self.exptime
    }

    pub fn set_exptime(&mut self, exptime: f64) {
        self.bump_cas_id();
        self.exptime = exptime;
    }

    pub fn get_atime(&self) -> &f64 {
        &self.atime
    }

    pub fn get_cas_id(&self) -> &u64 {
        &self.cas_id
    }

    fn bump_cas_id(&mut self) {
        self.cas_id += 1;
    }

    pub fn touch(&mut self) {
        self.atime = time_now();
    }


    pub fn len(&self) -> usize {
        self.item.len()
    }
}
