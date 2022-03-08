use std::net::SocketAddr;
use std::num::Wrapping;

use clap::Parser;

use tokio::{
    io::{AsyncWriteExt, Error},
    net::TcpStream,
    time::Instant,
};

const PATTERN: &[u8; 4] = b"SSH-";

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(short)]
    delay: u64,
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
