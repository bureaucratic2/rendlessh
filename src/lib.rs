use std::fs::read_to_string;
use std::num::Wrapping;
use std::path::Path;
use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

use tokio::{io::AsyncWriteExt, net::TcpStream, time::Instant};
use toml::Value;

use crate::error::Result;

const DEFAULT_PORT: u32 = 2222;
const DEFAULT_DELAY: u64 = 10000;
const DEFAULT_MAX_LEN: usize = 32;
const PATTERN: &[u8; 4] = b"SSH-";

#[derive(Clone, Copy)]
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
            if parse_toml(path, &mut config).is_err() {
                // todo log
            }
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
                Ok(())
            }
            Err(e) => Err(e.into()),
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

#[cfg(test)]
mod test {
    use std::fs::read_to_string;
    use toml::Value;
    #[test]
    fn parse_toml() {
        let path = "/home/user1/projects/rendlessh/example.toml";
        let content = read_to_string(path).unwrap();
        let val = content.parse::<Value>().unwrap();

        assert_eq!(val["Port"].as_integer(), Some(2222));
        assert_eq!(val["Delay"].as_integer(), Some(10000));
        if let Some(len) = val.get("MaxLineLength") {
            assert_eq!(len.as_integer(), Some(32));
        }
        // assert_eq!(val["MaxLineLength"].as_integer(), Some(32));
    }
}

mod error;
