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
pub enum SetInstr {
    Set, // Store an item
    Add, // Store only if the key does not yet exist
    Replace, // Store only if the key does already exist
    Append, // Append the data for an existing item
    Prepend, // Prepend the data for an existing item
    Cas, // Compare and swap
}


#[derive(Debug, PartialEq, Clone)]
pub struct Set {
    pub instr: SetInstr, // Instruction to perform
    pub key: String, // Alphanumeric characters
    pub flags: u16, // Arbitrary bit pattern chosen by the client
    pub exptime: u32, // Relative (secs) or absolute (unixtime) expiry time
    pub data: Vec<u8>, // Binary data
    pub noreply: bool, // Indicates whether the server should reply to the set
}

impl Set {
    pub fn new(instr: SetInstr,
               key: &str,
               flags: u16,
               exptime: u32,
               data: Vec<u8>,
               noreply: bool)
               -> Set {
        Set {
            instr: instr,
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
    pub flags: u16,
    pub data: Vec<u8>,
}

impl Value {
    pub fn new(key: &str, flags: u16, data: Vec<u8>) -> Value {
        Value {
            key: key.to_string(),
            flags: flags,
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
    // A sentinel value to indicate that there is nothing to return to the
    // client (in case of noreply)
    Empty,

    Error,
    ClientError(ClientError),
    ServerError(ServerError),

    Deleted, // The item was deleted successfully
    Exists, // The cas item has been modified
    NotFound, // The cas item does not exist
    NotStored, // Precondition not met
    Stored, // The item was stored successfully
    Touched, // The item was touched successfully

    Stats(Vec<Stat>),
    Value(Value),
}
