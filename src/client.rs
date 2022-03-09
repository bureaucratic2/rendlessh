use std::net::SocketAddr;
use std::num::Wrapping;

use log::{debug, info};
use tokio::sync::mpsc;
use tokio::{io::AsyncWriteExt, net::TcpStream, time::Instant};

use crate::{Result, StatisticEvent};

const PATTERN: &[u8; 4] = b"SSH-";

pub struct Client {
    addr: SocketAddr,
    connect_time: Instant,
    bytes_sent: u64,
    rng: u128,
    stream: TcpStream,

    pub tx: mpsc::Sender<StatisticEvent>,
}

impl Client {
    pub fn new(
        addr: SocketAddr,
        connect_time: Instant,
        stream: TcpStream,
        tx: mpsc::Sender<StatisticEvent>,
    ) -> Self {
        Self {
            addr,
            connect_time,
            bytes_sent: 0,
            // undetermined elapsed time as random generator seed
            rng: connect_time.elapsed().as_nanos(),
            stream,
            tx,
        }
    }

    pub async fn sendline(&mut self, max_len: usize) -> Result<()> {
        let line = randline(max_len, &mut self.rng);
        match self.stream.write_all(&line).await {
            Ok(_) => {
                self.bytes_sent += line.len() as u64;
                self.tx
                    .send(StatisticEvent::BytesSent(line.len()))
                    .await
                    .unwrap();
                debug!("{} bytes sent to {}", line.len(), self.addr);
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

impl Drop for Client {
    fn drop(&mut self) {}
}

fn rand16(rng: &mut u128) -> u128 {
    *rng = (Wrapping(*rng) * Wrapping(1103515245) + Wrapping(12345)).0;
    (*rng >> 16) & 0xfffff
}

fn randline(max_len: usize, rng: &mut u128) -> Vec<u8> {
    let len = 4 + rand16(rng) as usize % (max_len - 2);
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

    debug!("{:?}", std::str::from_utf8(&line).unwrap());

    line
}
