use std::fmt::{self, Display};
use std::fs::read_to_string;
use std::num::Wrapping;
use std::path::Path;
use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

use log::{debug, info};
use tokio::{io::AsyncWriteExt, net::TcpStream, time::Instant};
use toml::Value;

use crate::error::Result;
use crate::logger::setup_logging;

const DEFAULT_PORT: u32 = 2222;
const DEFAULT_DELAY: u64 = 10000;
const DEFAULT_MAX_LEN: usize = 32;
const PATTERN: &[u8; 4] = b"SSH-";

pub struct Config {
    pub port: u32,
    pub delay: u64,
    pub length: usize,
    path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            delay: DEFAULT_DELAY,
            length: DEFAULT_MAX_LEN,
            path: PathBuf::default(),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[Port {}] [Delay {}ms] [Max Length {}]",
            self.port, self.delay, self.length
        )
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

    /// Log level (repeatable)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u64,
}

impl Cli {
    pub fn parse_args() -> Config {
        let cli = Cli::parse();

        setup_logging(cli.verbose).expect("Fail to set global logger");

        let mut config = Config::default();

        if let Some(path) = cli.config {
            match parse_toml(&path, &mut config) {
                Ok(_) => info!("Load config from {:#?}, {}", path.as_os_str(), config),
                Err(err) => info!("{}", err.to_string()),
            }
            config.path = path;
        }

        if let Some(port) = cli.port {
            if port < 65536 {
                config.port = port;
            }
        }

        if let Some(delay) = cli.delay {
            if delay > 0 {
                config.delay = delay;
            }
        }

        if let Some(length) = cli.length {
            if length > 2 && length < 256 {
                config.length = length;
            }
        }

        info!("Current config {}", config);

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

    pub async fn sendline(&mut self, max_len: usize) -> Result<()> {
        let line = randline(max_len, &mut self.rng);
        match self.stream.write_all(&line).await {
            Ok(_) => {
                self.bytes_sent += line.len() as u64;
                debug!("Sent {} bytes to {}", line.len(), self.addr);
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn loginfo(&self) {
        info!(
            "connection from {} last {}s, {} bytes sent",
            self.addr,
            self.connect_time.elapsed().as_millis() as f64 / 1000.0,
            self.bytes_sent
        );
    }
}

fn rand16(rng: &mut u128) -> u128 {
    *rng = (Wrapping(*rng) * Wrapping(1103515245) + Wrapping(12345)).0;
    (*rng >> 16) & 0xfffff
}

fn randline(max_len: usize, rng: &mut u128) -> Vec<u8> {
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

fn parse_toml<P>(path: P, cfg: &mut Config) -> Result<()>
where
    P: AsRef<Path>,
{
    let content = read_to_string(path)?;
    let val = content.parse::<Value>()?;

    if let Some(port) = val.get("Port") {
        let port = port.as_integer().unwrap();
        if port > 0 && port < 65536 {
            cfg.port = port as u32;
        }
    }

    if let Some(delay) = val.get("Delay") {
        let delay = delay.as_integer().unwrap();
        if delay > 0 {
            cfg.delay = delay as u64;
        }
    }

    if let Some(len) = val.get("MaxLineLength") {
        let len = len.as_integer().unwrap();
        if len > 2 && len < 256 {
            cfg.length = len as usize;
        }
    }

    Ok(())
}

mod error;
mod logger;
