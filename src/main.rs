use log::info;
use rendlessh::{background_statistic, Cli, Client, Config, Result, StatisticEvent};

use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::{self, Instant};

use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

async fn handle_signals(mut signals: Signals, tx: oneshot::Sender<bool>) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM => {
                // gracefully shut down the daemon, allowing it to
                // write a complete, consistent log.
                let _ = tx.send(true);
                return;
            }
            SIGHUP => {
                // requests a reload of the configuration file
            }
            SIGUSR1 => {
                // print connections stats to the log
            }
            _ => unreachable!(),
        }
    }
}

async fn honeypot(mut client: Client, mut rx: watch::Receiver<Config>) {
    let delay;
    let mut len;
    {
        let cfg = rx.borrow();
        delay = cfg.delay;
        len = cfg.length;
    }
    let mut interval = time::interval(time::Duration::from_millis(delay));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if client.sendline(len).await.is_err() {
                    client.loginfo();
                    break;
                }
            }
            _ = rx.changed() => {
                let cfg = rx.borrow();
                len = cfg.length;
                if delay != cfg.delay {
                    interval = time::interval(time::Duration::from_millis(cfg.delay));
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Cli::parse_args();

    // register signal handlers
    let signals = Signals::new(&[SIGTERM, SIGHUP, SIGUSR1])?;
    let handle = signals.handle();
    let (sig_tx, mut sig_rx) = oneshot::channel();
    let signals_task = tokio::spawn(handle_signals(signals, sig_tx));

    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;
    // config channel, RW Lock
    let (cfg_tx, _) = watch::channel(config);

    // statistics
    let (stat_tx, stat_rx) = mpsc::channel(100);
    tokio::spawn(background_statistic(stat_rx));
    loop {
        tokio::select! {
            res = listener.accept() => {
                if let Ok((stream, addr)) = res {
                    let c = Client::new(addr, Instant::now(), stream, stat_tx.clone());
                    println!("accept from {}", addr);
                    stat_tx.send(StatisticEvent::NewConn).await.unwrap();
                    tokio::spawn(honeypot(c, cfg_tx.subscribe()));
                }
            }
            _ = &mut sig_rx => {
                break;
            }
        }
    }
    drop(stat_tx);

    handle.close();
    let _ = signals_task.await;
    info!("Gracefully exit");
    Ok(())
}
