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
    pub key: String,
    pub exptime: u64,
    pub data: Vec<u8>,
}

impl Set {
    pub fn new(key: &str, exptime: u64, data: Vec<u8>) -> Set {
        Set {
            key: key.to_string(),
            exptime: exptime,
            data: data,
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
pub struct Value {
    pub key: String,
    pub data: Vec<u8>,
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
    ClientError(ClientError),
    ServerError(ServerError),
    Error,

    Stored,
    Value(Value),
}
