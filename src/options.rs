use docopt::Docopt;


// Write the Docopt usage string.
const USAGE: &'static str = "
Usage:
    memcache [options]

Options:
    --host HOST         Interface to listen on (ie. ip hostname/ip).
    --port PORT         Port to bind to.
    --mem MEMSIZE       Max memory to use (in megabytes).
    --metrics           Collect server performance metrics.
    -h --help           Show this screen.
";


#[derive(Debug, Clone, RustcDecodable)]
pub struct MemcacheOptions {
    pub flag_host: Option<String>,
    pub flag_port: Option<u16>,
    pub flag_mem: Option<u64>,
    pub flag_metrics: bool,
}

impl MemcacheOptions {
    pub fn get_bind_params(&self) -> (String, u16) {
        let opts = self.clone();
        (opts.flag_host.unwrap().clone(),
         opts.flag_port.unwrap().clone())
    }

    pub fn get_bind_string(&self) -> String {
        let (host, port) = self.get_bind_params();
        format!("{}:{}", host, port)
    }

    pub fn get_mem_limit(&self) -> u64 {
        self.flag_mem.unwrap()
    }

    pub fn get_mem_limit_bytes(&self) -> u64 {
        self.flag_mem.unwrap() << 20
    }

    pub fn get_metrics_enabled(&self) -> bool {
        self.flag_metrics
    }
}


pub fn parse_args() -> MemcacheOptions {
    let mut opts: MemcacheOptions = Docopt::new(USAGE)
                                        .and_then(|d| d.decode())
                                        .unwrap_or_else(|e| e.exit());

    // println!("{:?}", opts);

    if opts.flag_host.is_none() {
        opts.flag_host = Some("127.0.0.1".to_string());
    }
    if opts.flag_port.is_none() {
        opts.flag_port = Some(11311);
    }

    if opts.flag_mem.is_none() {
        opts.flag_mem = Some(64);
    }

    opts
}
