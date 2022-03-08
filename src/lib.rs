use std::num::Wrapping;
use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

use tokio::{
    io::{AsyncWriteExt, Error},
    net::TcpStream,
    time::Instant,
};

const DEFAULT_PORT: u32 = 2222;
const DEFAULT_DELAY: u64 = 10000;
const DEFAULT_MAX_LEN: usize = 32;
const PATTERN: &[u8; 4] = b"SSH-";

pub struct Config {
    pub port: u32,
    pub delay: u64,
    pub length: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            delay: DEFAULT_DELAY,
            length: DEFAULT_MAX_LEN,
        }
    }
}

/// Hello
#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// Message millisecond delay [10000]
    #[clap(short, long, value_name("NON-ZERO UINT"))]
    delay: Option<u64>,

    /// Sets a custom config file
    #[clap(short, long, value_name("FILE"))]
    config: Option<PathBuf>,

    /// Maximum banner line length (3-255) [32]
    #[clap(short, long, value_name("UINT"))]
    length: Option<usize>,

    /// Listening port [2222]
    #[clap(short, long, value_name("UINT"))]
    port: Option<u32>,
}

impl Cli {
    pub fn parse_args() -> Config {
        let cli = Cli::parse();
        let mut config = Config::default();

        if let Some(path) = cli.config.as_deref() {
            // parse toml
        }

        if let Some(port) = cli.port {
            config.port = port;
        }

        if let Some(delay) = cli.delay {
            if delay == 0 {
                // todo
                panic!()
            }
            config.delay = delay;
        }

        if let Some(length) = cli.length {
            if length < 3 || length > 255 {
                // todo
                panic!()
            }
            config.length = length;
        }

        config
    }
}

pub struct Client {
    addr: SocketAddr,
    connect_time: Instant,
    bytes_sent: u64,
    rng: u128,
    stream: TcpStream,
}

impl Client {
    pub fn new(addr: SocketAddr, connect_time: Instant, stream: TcpStream) -> Self {
        Self {
            addr,
            connect_time,
            bytes_sent: 0,
            // undetermined elapsed time as random generator seed
            rng: connect_time.elapsed().as_nanos(),
            stream,
        }
    }

    pub async fn sendline(&mut self, max_len: usize) -> Result<(), Error> {
        let line = randline(max_len, &mut self.rng);
        match self.stream.write_all(&line).await {
            Ok(_) => {
                self.bytes_sent += line.len() as u64;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn loginfo(&self) {
        println!(
            "connection from {} last {}ms, sent {} bytes",
            self.addr,
            self.connect_time.elapsed().as_millis(),
            self.bytes_sent
        );
    }
}

pub fn rand16(rng: &mut u128) -> u128 {
    *rng = (Wrapping(*rng) * Wrapping(1103515245) + Wrapping(12345)).0;
    (*rng >> 16) & 0xfffff
}

pub fn randline(max_len: usize, rng: &mut u128) -> Vec<u8> {
    let len = 3 + rand16(rng) as usize % (max_len - 2);
    let mut line = vec![0u8; len];

    for ch in line.iter_mut().take(len - 2) {
        // ASCII 32~127, printable characters
        *ch = 32 + (rand16(rng) % 95) as u8;
    }

    // /r/n
    line[len - 2] = 13;
    line[len - 1] = 10;
    if &line[0..4] == PATTERN {
        line[0] = b'X';
    }

    line
}
