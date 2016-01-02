// ref: https://github.com/memcached/memcached/blob/master/doc/protocol.txt


// Request structs

#[derive(Debug, PartialEq, Clone)]
pub struct Get {
    pub key: String,
}

impl Get {
    pub fn new(key: &str) -> Get {
        Get { key: key.to_string() }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct Set {
    pub key: String, // alphanumeric characters
    pub flags: u16, // arbitrary bit pattern chosen by the client
    pub exptime: u32, // relative (secs) or absolute (unixtime) expiry time
    pub data: Vec<u8>, // binary data
    pub noreply: bool, // indicates whether the server should reply to the set
}

impl Set {
    pub fn new(key: &str,
               flags: u16,
               exptime: u32,
               data: Vec<u8>,
               noreply: bool)
               -> Set {
        Set {
            key: key.to_string(),
            flags: flags,
            exptime: exptime,
            data: data,
            noreply: noreply,
        }
    }
}


// Response structs

#[derive(Debug, PartialEq, Clone)]
pub struct ClientError {
    pub error: String,
}


#[derive(Debug, PartialEq, Clone)]
pub struct ServerError {
    pub error: String,
}


#[derive(Debug, PartialEq, Clone)]
pub struct Stat {
    pub key: String,
    pub value: String,
}

impl Stat {
    pub fn new(key: &str, value: String) -> Stat {
        Stat {
            key: key.to_string(),
            value: value,
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct Value {
    pub key: String,
    pub data: Vec<u8>,
}

impl Value {
    pub fn new(key: &str, data: Vec<u8>) -> Value {
        Value {
            key: key.to_string(),
            data: data,
        }
    }
}


// High level groupings

#[derive(Debug, PartialEq, Clone)]
pub enum Cmd {
    Get(Get),
    Set(Set),
    Stats,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Resp {
    Error,
    ClientError(ClientError),
    ServerError(ServerError),

    Stats(Vec<Stat>),
    Stored,
    Value(Value),
}
