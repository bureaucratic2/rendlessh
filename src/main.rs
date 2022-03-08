use rendlessh::{Cli, Client};

use tokio::net::TcpListener;
use tokio::time::{self, Instant};

const DEFAULT_PORT: u32 = 2222;
const DEFAULT_DELAY: u64 = 10000;
const DEFAULT_MAX_LEN: usize = 32;

async fn honeypot(mut client: Client) {
    let mut interval = time::interval(time::Duration::from_millis(DEFAULT_DELAY));

    loop {
        interval.tick().await;
        if client.sendline(DEFAULT_MAX_LEN).await.is_err() {
            client.loginfo();
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Cli::parse_args();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", DEFAULT_PORT))
        .await
        .unwrap();
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        let c = Client::new(addr, Instant::now(), stream);
        println!("accept from {}", addr);
        tokio::spawn(honeypot(c));
    }
}
