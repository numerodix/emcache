use storage::Cache;
use storage::Key;
use storage::Value;

use super::cmd::Cmd;
use super::cmd::Get;
use super::cmd::Resp;
use super::cmd::Set;
use super::cmd::Stat;
use super::cmd::Value as CmdValue;


pub struct Driver {
    cache: Cache,
}

impl Driver {
    pub fn new(cache: Cache) -> Driver {
        Driver { cache: cache }
    }


    fn do_get(&mut self, get: Get) -> Resp {
        // XXX get rid of all the cloning
        let get_clone = get.clone();
        let key = Key::new(get.key.into_bytes());

        let rv = self.cache.get(&key);

        match rv {
            Ok(value) => {
                Resp::Value(CmdValue {
                    key: get_clone.key,
                    data: value.item.clone(),
                })
            }
            Err(_) => Resp::Error,
        }
    }

    fn do_set(&mut self, set: Set) -> Resp {
        let key = Key::new(set.key.into_bytes());
        let value = Value::new(set.data);
        // XXX handle set.exptime

        let rv = self.cache.set(key, value);

        match rv {
            Ok(_) => Resp::Stored,
            Err(_) => Resp::Error,
        }
    }

    fn do_stats(&self) -> Resp {
        let curr_items = self.cache.len();

        let stat = Stat::new("curr_items", curr_items.to_string());
        Resp::Stats(vec![stat])
    }


    pub fn run(&mut self, cmd: Cmd) -> Resp {
        match cmd {
            Cmd::Get(get) => self.do_get(get),
            Cmd::Set(set) => self.do_set(set),
            Cmd::Stats => self.do_stats(),
        }
    }
}
