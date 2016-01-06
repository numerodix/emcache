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
use super::cmd::FlushAll;
use super::cmd::Get;
use super::cmd::GetInstr;
use super::cmd::Inc;
use super::cmd::IncInstr;
use super::cmd::Resp;
use super::cmd::Set;
use super::cmd::SetInstr;
use super::cmd::Stat;
use super::cmd::Touch;
use super::cmd::Value as CmdValue;
use super::util::bytes_to_u64;
use super::util::convert_exptime;
use super::util::from_cache_err;
use super::util::u64_to_bytes;


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
    incr_hits: u64,
    incr_misses: u64,
    decr_hits: u64,
    decr_misses: u64,
    cas_misses: u64,
    cas_hits: u64,
    cas_badval: u64,
    touch_misses: u64,
    touch_hits: u64,
}

impl DriverStats {
    pub fn new() -> DriverStats {
        DriverStats {
            cmd_get: 0,
            cmd_set: 0,
            cmd_flush: 0,
            cmd_touch: 0,
            incr_hits: 0,
            incr_misses: 0,
            decr_hits: 0,
            decr_misses: 0,
            cas_misses: 0,
            cas_hits: 0,
            cas_badval: 0,
            touch_hits: 0,
            touch_misses: 0,
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
        match convert_exptime(exptime) {
            Some(tm) => {
                value.set_exptime(tm);
            }
            None => (),
        }
    }


    fn do_add(&mut self, set: Set) -> Resp {
        let key = Key::new(set.key.into_bytes());

        // Do we store this item already? If so it's an early exit.
        let rv = self.cache.contains_key(&key);
        maybe_reply_stmt!(!set.noreply,
                          match rv {
                              Ok(true) => Some(Resp::NotStored),
                              Ok(false) => None,
                              Err(ref err) => Some(from_cache_err(err)),
                          });

        let mut value = Value::new(set.data);
        value.set_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!set.noreply,
                          match rv {
                              Ok(_) => Resp::Stored,
                              Err(ref err) => from_cache_err(err),
                          })
    }

    fn do_append(&mut self, set: Set) -> Resp {
        let key = Key::new(set.key.into_bytes());

        // Load the value
        let rv = self.cache.remove(&key);

        // If if it's not there we error out
        maybe_reply_stmt!(!set.noreply,
                          match rv {
                              Err(CacheError::KeyNotFound) => {
                                  Some(Resp::NotStored)
                              }
                              Err(ref err) => Some(from_cache_err(err)),
                              _ => None,
                          });

        // Update the value
        let mut value = rv.unwrap();
        value.set_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        // Append the data we just received to the blob that is there
        {
            let mut blob = value.get_item_mut();
            blob.extend(set.data);
        }

        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!set.noreply,
                          match rv {
                              Ok(_) => Resp::Stored,
                              Err(ref err) => from_cache_err(err),
                          })
    }

    fn do_cas(&mut self, set: Set) -> Resp {
        let key = Key::new(set.key.into_bytes());

        // If the key is not set we bail
        let rv = self.cache.contains_key(&key);
        maybe_reply_stmt!(!set.noreply,
                          match rv {
                              Ok(true) => {
                                  // Update stats
                                  self.stats.cas_hits += 1;

                                  None
                              }
                              Ok(false) => {
                                  // Update stats
                                  self.stats.cas_misses += 1;

                                  Some(Resp::NotFound)
                              }
                              Err(ref err) => Some(from_cache_err(err)),
                          });

        // If cas_unique is out of date we bail
        maybe_reply_stmt!(!set.noreply, {
            let rv = self.cache.get(&key);
            let value = rv.unwrap();

            match *value.get_cas_id() == set.cas_unique.unwrap() {
                true => None,
                false => {
                    // Update stats
                    self.stats.cas_badval += 1;

                    Some(Resp::Exists)
                }
            }
        });

        // Load the value
        let rv = self.cache.remove(&key);
        let mut value = rv.unwrap();

        // Set all the data the client sent
        value.set_item(set.data);
        value.set_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!set.noreply,
                          match rv {
                              Ok(_) => Resp::Stored,
                              Err(ref err) => from_cache_err(err),
                          })
    }

    fn do_delete(&mut self, delete: Delete) -> Resp {
        let key = Key::new(delete.key.clone().into_bytes());

        let rv = self.cache.remove(&key);

        maybe_reply_expr!(!delete.noreply,
                          match rv {
                              Ok(_) => Resp::Deleted,
                              Err(CacheError::KeyNotFound) => Resp::NotFound,
                              Err(ref err) => from_cache_err(err),
                          })
    }

    fn do_flush_all(&mut self, flush_all: FlushAll) -> Resp {
        // Update stats
        self.stats.cmd_flush += 1;

        let exptime: f64 = match flush_all.exptime {
            Some(ref exptime) => convert_exptime(*exptime).unwrap(),
            None => time_now(),
        };

        let rv = self.cache.flush_all(exptime);

        maybe_reply_expr!(!flush_all.noreply,
                          match rv {
                              Ok(_) => Resp::Ok,
                              Err(ref err) => from_cache_err(err),
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
                    let mut val_st = CmdValue {
                        key: key_str,
                        flags: value.get_flags().clone(),
                        cas_unique: None,
                        data: value.get_item().clone(),
                    };

                    if get.instr == GetInstr::Gets {
                        val_st.with_cas_unique(value.get_cas_id().clone());
                    }

                    values.push(val_st);
                }
                // Keys that were not found are skipped, no error given
                Err(_) => (),
            }
        }

        Resp::Values(values)
    }

    fn do_inc(&mut self, inc: Inc) -> Resp {
        let key = Key::new(inc.key.clone().into_bytes());

        {
            // Check the value first
            let rv = self.cache.get(&key);
            maybe_reply_stmt!(!inc.noreply,
                              match rv {
                                  Ok(_) => {
                                      // Update stats
                                      match inc.instr {
                                          IncInstr::Decr => {
                                              self.stats.decr_hits += 1;
                                          }
                                          IncInstr::Incr => {
                                              self.stats.incr_hits += 1;
                                          }
                                      };

                                      None
                                  }
                                  Err(CacheError::KeyNotFound) => {
                                      // Update stats
                                      match inc.instr {
                                          IncInstr::Decr => {
                                              self.stats.decr_misses += 1;
                                          }
                                          IncInstr::Incr => {
                                              self.stats.incr_misses += 1;
                                          }
                                      };

                                      Some(Resp::NotFound)
                                  }
                                  Err(ref err) => Some(from_cache_err(err)),
                              });

            let value = rv.unwrap();
            // Does it represent a number?
            maybe_reply_stmt!(!inc.noreply,
                              match bytes_to_u64(value.get_item()) {
                                  Some(_) => None,
                                  None => {
                                      Some(Resp::ClientError("Not a number"
                                                                 .to_string()))
                                  }
                              });
        }

        // Update the value
        let rv = self.cache.remove(&key);
        let mut value = rv.unwrap();

        // Apply incr/decr
        let mut num = bytes_to_u64(value.get_item()).unwrap();
        match inc.instr {
            IncInstr::Decr => {
                // saturates (stays at 0), does not underflow
                num = num.saturating_sub(inc.delta);
            }
            IncInstr::Incr => {
                // overflows
                num = num.wrapping_add(inc.delta);
            }
        };
        value.set_item(u64_to_bytes(&num));

        // Set it
        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!inc.noreply,
                          match rv {
                              Ok(_) => Resp::IntValue(num),
                              Err(ref err) => from_cache_err(err),
                          })
    }

    fn do_prepend(&mut self, set: Set) -> Resp {
        let key = Key::new(set.key.into_bytes());

        // Load the value
        let rv = self.cache.remove(&key);

        // If if it's not there we error out
        maybe_reply_stmt!(!set.noreply,
                          match rv {
                              Ok(_) => None,
                              Err(CacheError::KeyNotFound) => {
                                  Some(Resp::NotStored)
                              }
                              Err(ref err) => Some(from_cache_err(err)),
                          });

        // Update the value
        let mut value = rv.unwrap();
        value.set_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        // Prepend the data we just received to the blob that is there
        let mut new_item = Vec::with_capacity(set.data.len() +
                                              value.get_item().len());
        new_item.extend(set.data);
        new_item.extend(value.get_item());
        value.set_item(new_item);

        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!set.noreply,
                          match rv {
                              Ok(_) => Resp::Stored,
                              Err(ref err) => from_cache_err(err),
                          })
    }

    fn do_replace(&mut self, set: Set) -> Resp {
        let key = Key::new(set.key.into_bytes());

        // Do we store this item already? If not it's an early exit.
        let rv = self.cache.contains_key(&key);
        maybe_reply_stmt!(!set.noreply,
                          match rv {
                              Ok(true) => None,
                              Ok(false) => Some(Resp::NotStored),
                              Err(ref err) => Some(from_cache_err(err)),
                          });

        let mut value = Value::new(set.data);
        value.set_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!set.noreply,
                          match rv {
                              Ok(_) => Resp::Stored,
                              Err(ref err) => from_cache_err(err),
                          })
    }

    fn do_set(&mut self, set: Set) -> Resp {
        // Update stats
        self.stats.cmd_set += 1;

        let key = Key::new(set.key.into_bytes());

        // Obtain either the existing value or a fresh one
        let mut value = {
            let rv = self.cache.remove(&key);
            match rv {
                Ok(_) => rv.unwrap(),
                Err(_) => Value::empty(),
            }
        };

        // Set all the data the client sent
        value.set_item(set.data);
        value.set_flags(set.flags);
        self.set_exptime(&mut value, set.exptime);

        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!set.noreply,
                          match rv {
                              Ok(_) => Resp::Stored,
                              Err(ref err) => from_cache_err(err),
                          })
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
        let incr_hits = self.stats.incr_hits.to_string();
        let incr_misses = self.stats.incr_misses.to_string();
        let decr_hits = self.stats.decr_hits.to_string();
        let decr_misses = self.stats.decr_misses.to_string();
        let cas_hits = self.stats.cas_hits.to_string();
        let cas_misses = self.stats.cas_misses.to_string();
        let cas_badval = self.stats.cas_badval.to_string();
        let touch_hits = self.stats.touch_hits.to_string();
        let touch_misses = self.stats.touch_misses.to_string();
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
        let st_incr_hits = Stat::new("incr_hits", incr_hits);
        let st_incr_misses = Stat::new("incr_misses", incr_misses);
        let st_decr_hits = Stat::new("decr_hits", decr_hits);
        let st_decr_misses = Stat::new("decr_misses", decr_misses);
        let st_cas_hits = Stat::new("cas_hits", cas_hits);
        let st_cas_misses = Stat::new("cas_misses", cas_misses);
        let st_cas_badval = Stat::new("cas_badval", cas_badval);
        let st_touch_hits = Stat::new("touch_hits", touch_hits);
        let st_touch_misses = Stat::new("touch_misses", touch_misses);
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
                         st_incr_hits,
                         st_incr_misses,
                         st_decr_hits,
                         st_decr_misses,
                         st_cas_hits,
                         st_cas_misses,
                         st_cas_badval,
                         st_touch_hits,
                         st_touch_misses,
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
        // Update stats
        self.stats.cmd_touch += 1;

        let key = Key::new(touch.key.into_bytes());

        // See if the key is set
        let rv = self.cache.contains_key(&key);

        // If if it's not there we error out
        maybe_reply_stmt!(!touch.noreply,
                          match rv {
                              Ok(true) => {
                                  // Update stats
                                  self.stats.touch_hits += 1;

                                  None
                              }
                              Ok(false) => {
                                  // Update stats
                                  self.stats.touch_misses += 1;

                                  Some(Resp::NotFound)
                              }
                              Err(ref err) => Some(from_cache_err(err)),
                          });

        // Load the value
        let rv = self.cache.remove(&key);
        maybe_reply_stmt!(!touch.noreply,
                          match rv {
                              Ok(_) => None,
                              Err(ref err) => Some(from_cache_err(err)),
                          });

        // Update the value
        let mut value = rv.unwrap();
        self.set_exptime(&mut value, touch.exptime);

        // Set it
        let rv = self.cache.set(key, value);

        maybe_reply_expr!(!touch.noreply,
                          match rv {
                              Ok(_) => Resp::Touched,
                              Err(ref err) => from_cache_err(err),
                          })
    }

    pub fn do_version(&self) -> Resp {
        Resp::Version(get_version_string())
    }


    pub fn run(&mut self, cmd: Cmd) -> Resp {
        match cmd {
            Cmd::Delete(del) => self.do_delete(del),
            Cmd::FlushAll(flush_all) => self.do_flush_all(flush_all),
            Cmd::Get(get) => self.do_get(get),
            Cmd::Inc(inc) => self.do_inc(inc),
            Cmd::Quit => Resp::Empty,  // handled at transport level
            Cmd::Set(set) => {
                match set.instr {
                    SetInstr::Add => self.do_add(set),
                    SetInstr::Append => self.do_append(set),
                    SetInstr::Prepend => self.do_prepend(set),
                    SetInstr::Replace => self.do_replace(set),
                    SetInstr::Set => self.do_set(set),
                    SetInstr::Cas => self.do_cas(set),
                }
            }
            Cmd::Stats => self.do_stats(),
            Cmd::Touch(touch) => self.do_touch(touch),
            Cmd::Version => self.do_version(),
        }
    }

    pub fn update_transport_stats(&mut self, stats: TransportStats) {
        self.transport_stats = stats;
    }
}
