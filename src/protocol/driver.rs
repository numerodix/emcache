use platform::process::get_pid;
use platform::time::time_now;
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
    time_start: f64,
}

impl Driver {
    pub fn new(cache: Cache) -> Driver {
        Driver {
            cache: cache,
            time_start: time_now(),
        }
    }


    fn set_exptime(&self, value: &mut Value, exptime: u32) {
        // If exptime is greater than zero we need to set it on the value
        if exptime > 0 {
            let tm;

            // Is it an interval greater than 30 days? Then it's a timestamp
            if exptime > 60 * 60 * 24 * 30 {
                tm = exptime as f64;

            } else {
                // Otherwise it's relative from now
                tm = time_now() + exptime as f64;
            }

            value.set_exptime(tm);
        }
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
        let mut value = Value::new(set.data);
        self.set_exptime(&mut value, set.exptime);

        let rv = self.cache.set(key, value);

        match rv {
            Ok(_) => Resp::Stored,
            Err(_) => Resp::Error,
        }
    }

    fn do_stats(&self) -> Resp {
        let stats = self.cache.get_stats();

        let pid = get_pid().to_string();
        let uptime = ((time_now() - self.time_start) as u64).to_string();
        let time = (time_now() as u64).to_string();
        let bytes = stats.bytes.to_string();
        let curr_items = self.cache.len().to_string();
        let total_items = stats.total_items.to_string();

        let st_pid = Stat::new("pid", pid);
        let st_uptime = Stat::new("uptime", uptime);
        let st_time = Stat::new("time", time);
        let st_bytes = Stat::new("bytes", bytes);
        let st_curr_items = Stat::new("curr_items", curr_items);
        let st_total_items = Stat::new("total_items", total_items);

        Resp::Stats(vec![st_pid,
                         st_uptime,
                         st_time,
                         st_bytes,
                         st_curr_items,
                         st_total_items])
    }


    pub fn run(&mut self, cmd: Cmd) -> Resp {
        match cmd {
            Cmd::Get(get) => self.do_get(get),
            Cmd::Set(set) => self.do_set(set),
            Cmd::Stats => self.do_stats(),
        }
    }
}
