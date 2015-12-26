use libc;


pub fn get_pid() -> u32 {
    unsafe { libc::getpid() as u32 }
}
