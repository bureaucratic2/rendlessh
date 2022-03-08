use rendlessh::{Cli, Client, Config};

use tokio::net::TcpListener;
use tokio::sync::watch::{self, Receiver};
use tokio::time::{self, Instant};

async fn honeypot(mut client: Client, rx: Receiver<Config>) {
    let delay;
    let len;
    {
        let cfg = rx.borrow();
        delay = cfg.delay;
        len = cfg.length;
    }
    let mut interval = time::interval(time::Duration::from_millis(delay));

    loop {
        interval.tick().await;
        if client.sendline(len).await.is_err() {
            client.loginfo();
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Cli::parse_args();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port))
        .await
        .unwrap();
    let (tx, _rx) = watch::channel(config);
    loop {
        if let Ok((stream, addr)) = listener.accept().await {
            let c = Client::new(addr, Instant::now(), stream);
            println!("accept from {}", addr);
            tokio::spawn(honeypot(c, tx.subscribe()));
        }
    }
}
