use common::consts::get_version_string;
use platform::process::get_pid;
use platform::time::time_now;
use storage::Cache;
use storage::CacheError;
use storage::Key;
use storage::Value;
use tcp_transport::stats::TransportStats;

use super::cmd::Cmd;
use super::cmd::Delete;
use super::cmd::Get;
use super::cmd::Resp;
use super::cmd::Set;
use super::cmd::SetInstr;
use super::cmd::Stat;
use super::cmd::Touch;
use super::cmd::Value as CmdValue;


// For use to get an early exit from a function. The first parameter is a bool
// to indicate whether to omit responses (returns Resp::Empty instead). The
// second parameter is an expression that evaluates to Option<Resp>. In case
// of None no return is made ie. execution continues. In case of Some(resp) we
// perform return resp;
//
// maybe_reply_stmt!(!noreply, match rv {
//     Ok(_) => None,   // do not return
//     Err(_) => Some(Resp::Error),
// });
//
// ->
//
// let opt = expr_provided;
// if !opt.is_none() {
//     if !noreply {
//         return opt.unwrap();
//     } else {
//         return Resp::Empty;
//     }
// }
macro_rules! maybe_reply_stmt {
    ( $cond:expr, $opt:expr ) => {
        {
            if !$opt.is_none() {
                if $cond {
                    return $opt.unwrap();
                } else {
                    return Resp::Empty;
                }
            };
        }
    };
}


// For use to conditionally produce a return value. The first parameter is used
// to indicate whether to omit responses (returns Resp::Empty instead). The
// second parameter is an expression that evaluates to Resp.
//
// maybe_reply_expr!(!noreply, match rv {
//     Ok(_) => Resp::Stored,
//     Err(_) => Resp::Error,
// });
//
// ->
//
// let resp = expr_provided;
// if !noreply {
//     resp
// } else {
//     Resp::Empty
// }
macro_rules! maybe_reply_expr {
    ( $cond:expr, $resp:expr ) => {
        {
            if $cond {
                $resp
            } else {
                Resp::Empty
            }
        }
    };
}


struct DriverStats {
    cmd_get: u64,
    cmd_set: u64,
    cmd_flush: u64,
    cmd_touch: u64,
}

impl DriverStats {
    pub fn new() -> DriverStats {
        DriverStats {
            cmd_get: 0,
            cmd_set: 0,
            cmd_flush: 0,
            cmd_touch: 0,
        }
    }
}


pub struct Driver {
    cache: Cache,
    time_start: f64,

    stats: DriverStats,
    transport_stats: TransportStats, // this is a global snapshot
}

impl Driver {
    pub fn new(cache: Cache) -> Driver {
        Driver {
            cache: cache,
            stats: DriverStats::new(),
            time_start: time_now(),
            transport_stats: TransportStats::new(),
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


    fn do_add(&mut self, set: Set) -> Resp {
        let key = Key::new(set.key.into_bytes());

        // Do we store this item already? If so it's an early exit.
        let rv = self.cache.contains_key(&key);
        maybe_reply_stmt!(!set.noreply, match rv {
            Ok(true) => {
                Some(Resp::NotStored)
            },
            Ok(false) => None,
            Err(_) => {
                Some(Resp::Error)
            }
        });

        let mut value = Value::new(set.data);
        value.with_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!set.noreply, match rv {
            Ok(_) => Resp::Stored,
            Err(_) => Resp::Error,
        })
    }

    fn do_append(&mut self, set: Set) -> Resp {
        let key = Key::new(set.key.into_bytes());

        // Load the value
        let rv = self.cache.remove(&key);

        // If if it's not there we error out
        maybe_reply_stmt!(!set.noreply, match rv {
            Err(CacheError::KeyNotFound) => Some(Resp::NotStored),
            Err(_) => Some(Resp::Error),
            _ => None,
        });

        // Update the value
        let mut value = rv.unwrap();
        value.with_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        // Append the data we just received to the blob that is there
        value.item.extend(set.data);

        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!set.noreply, match rv {
            Ok(_) => Resp::Stored,
            Err(_) => Resp::Error,
        })
    }

    fn do_delete(&mut self, delete: Delete) -> Resp {
        let key = Key::new(delete.key.clone().into_bytes());

        let rv = self.cache.remove(&key);

        maybe_reply_expr!(!delete.noreply, match rv {
            Ok(_) => Resp::Deleted,
            Err(_) => Resp::NotFound,
        })
    }

    fn do_get(&mut self, get: Get) -> Resp {
        // Update stats
        self.stats.cmd_get += 1;

        let mut values = vec![];

        for key in get.keys {
            let key_str = key.clone();

            let key_st = Key::new(key.into_bytes());
            let rv = self.cache.get(&key_st);

            match rv {
                Ok(value) => {
                    let val_st = CmdValue {
                        key: key_str,
                        flags: value.flags,
                        data: value.item.clone(),
                    };
                    values.push(val_st);
                }
                // Keys that were not found are skipped, no error given
                Err(_) => (),
            }
        }

        Resp::Values(values)
    }

    fn do_prepend(&mut self, set: Set) -> Resp {
        let key = Key::new(set.key.into_bytes());

        // Load the value
        let rv = self.cache.remove(&key);

        // If if it's not there we error out
        maybe_reply_stmt!(!set.noreply, match rv {
            Ok(_) => None,
            Err(CacheError::KeyNotFound) => Some(Resp::NotStored),
            Err(_) => Some(Resp::Error),
        });

        // Update the value
        let mut value = rv.unwrap();
        value.with_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        // Prepend the data we just received to the blob that is there
        let mut new_item = Vec::with_capacity(set.data.len() + value.item.len());
        new_item.extend(set.data);
        new_item.extend(value.item);
        value.item = new_item;

        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!set.noreply, match rv {
            Ok(_) => Resp::Stored,
            Err(_) => Resp::Error,
        })
    }

    fn do_replace(&mut self, set: Set) -> Resp {
        let key = Key::new(set.key.into_bytes());

        // Do we store this item already? If not it's an early exit.
        let rv = self.cache.contains_key(&key);
        maybe_reply_stmt!(!set.noreply, match rv {
            Ok(true) => None,
            Ok(false) => Some(Resp::NotStored),
            Err(_) => Some(Resp::Error),
        });

        let mut value = Value::new(set.data);
        value.with_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!set.noreply, match rv {
            Ok(_) => Resp::Stored,
            Err(_) => Resp::Error,
        })
    }

    fn do_set(&mut self, set: Set) -> Resp {
        // Update stats
        self.stats.cmd_set += 1;

        let key = Key::new(set.key.into_bytes());
        let mut value = Value::new(set.data);
        value.with_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        let rv = self.cache.set(key, value);

        match set.noreply {
            true => Resp::Empty,
            false => {
                match rv {
                    Ok(_) => Resp::Stored,
                    Err(_) => Resp::Error,
                }
            }
        }
    }

    fn do_stats(&self) -> Resp {
        let storage = self.cache.get_stats();

        let pid = get_pid().to_string();
        let uptime = ((time_now() - self.time_start) as u64).to_string();
        let time = (time_now() as u64).to_string();
        let version = get_version_string();
        let cmd_get = self.stats.cmd_get.to_string();
        let cmd_set = self.stats.cmd_set.to_string();
        let cmd_flush = self.stats.cmd_flush.to_string();
        let cmd_touch = self.stats.cmd_touch.to_string();
        let get_hits = storage.get_hits.to_string();
        let get_misses = storage.get_misses.to_string();
        let delete_hits = storage.delete_hits.to_string();
        let delete_misses = storage.delete_misses.to_string();
        let bytes_read = self.transport_stats.bytes_read.to_string();
        let bytes_written = self.transport_stats.bytes_written.to_string();
        let limit_maxbytes = self.cache.capacity.to_string();
        let bytes = storage.bytes.to_string();
        let curr_items = self.cache.len().to_string();
        let total_items = storage.total_items.to_string();
        let evictions = storage.evictions.to_string();
        let reclaimed = storage.reclaimed.to_string();

        let st_pid = Stat::new("pid", pid);
        let st_uptime = Stat::new("uptime", uptime);
        let st_time = Stat::new("time", time);
        let st_version = Stat::new("version", version);
        let st_cmd_get = Stat::new("cmd_get", cmd_get);
        let st_cmd_set = Stat::new("cmd_set", cmd_set);
        let st_cmd_flush = Stat::new("cmd_flush", cmd_flush);
        let st_cmd_touch = Stat::new("cmd_touch", cmd_touch);
        let st_get_hits = Stat::new("get_hits", get_hits);
        let st_get_misses = Stat::new("get_misses", get_misses);
        let st_delete_hits = Stat::new("delete_hits", delete_hits);
        let st_delete_misses = Stat::new("delete_misses", delete_misses);
        let st_bytes_read = Stat::new("bytes_read", bytes_read);
        let st_bytes_written = Stat::new("bytes_written", bytes_written);
        let st_limit_maxbytes = Stat::new("limit_maxbytes", limit_maxbytes);
        let st_bytes = Stat::new("bytes", bytes);
        let st_curr_items = Stat::new("curr_items", curr_items);
        let st_total_items = Stat::new("total_items", total_items);
        let st_evictions = Stat::new("evictions", evictions);
        let st_reclaimed = Stat::new("reclaimed", reclaimed);

        Resp::Stats(vec![st_pid,
                         st_uptime,
                         st_time,
                         st_version,
                         st_cmd_get,
                         st_cmd_set,
                         st_cmd_flush,
                         st_cmd_touch,
                         st_get_hits,
                         st_get_misses,
                         st_delete_hits,
                         st_delete_misses,
                         st_bytes_read,
                         st_bytes_written,
                         st_limit_maxbytes,
                         st_bytes,
                         st_curr_items,
                         st_total_items,
                         st_evictions,
                         st_reclaimed])
    }

    pub fn do_touch(&mut self, touch: Touch) -> Resp {
        let key = Key::new(touch.key.into_bytes());

        // See if the key is set
        let rv = self.cache.contains_key(&key);

        // If if it's not there we error out
        maybe_reply_stmt!(!touch.noreply, match rv {
            Ok(true) => None,
            Ok(false) => Some(Resp::NotFound),
            Err(_) => Some(Resp::Error),
        });

        // Load the value
        let rv = self.cache.remove(&key);
        maybe_reply_stmt!(!touch.noreply, match rv {
            Ok(_) => None,
            Err(_) => Some(Resp::Error),
        });

        // Update the value
        let mut value = rv.unwrap();
        self.set_exptime(&mut value, touch.exptime);

        // Set it
        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!touch.noreply, match rv {
            Ok(_) => Resp::Touched,
            Err(_) => Resp::Error,
        })
    }

    pub fn do_version(&self) -> Resp {
        Resp::Version(get_version_string())
    }


    pub fn run(&mut self, cmd: Cmd) -> Resp {
        match cmd {
            Cmd::Delete(del) => self.do_delete(del),
            Cmd::Get(get) => self.do_get(get),
            Cmd::Quit => Resp::Empty,  // handled at transport level
            Cmd::Set(set) => match set.instr {
                SetInstr::Add => self.do_add(set),
                SetInstr::Append => self.do_append(set),
                SetInstr::Prepend => self.do_prepend(set),
                SetInstr::Replace => self.do_replace(set),
                SetInstr::Set => self.do_set(set),
                _ => Resp::Error,  // TODO
            },
            Cmd::Stats => self.do_stats(),
            Cmd::Touch(touch) => self.do_touch(touch),
            Cmd::Version => self.do_version(),
        }
    }

    pub fn update_transport_stats(&mut self, stats: TransportStats) {
        self.transport_stats = stats;
    }
}
