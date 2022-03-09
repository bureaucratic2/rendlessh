use log::info;
use rendlessh::{background_statistic, reload_config, Cli, Client, Config, Result, StatisticEvent};

use tokio::net::TcpListener;
use tokio::sync::{mpsc, watch};
use tokio::time::{self, Instant};

use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

enum Sig {
    Exit,
    Reload,
    Log,
}

async fn handle_signals(mut signals: Signals, tx: mpsc::UnboundedSender<Sig>) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM => {
                // gracefully shut down the daemon, allowing it to
                // write a complete, consistent log.
                let _ = tx.send(Sig::Exit);
                return;
            }
            SIGHUP => {
                // requests a reload of the configuration file
                let _ = tx.send(Sig::Reload);
            }
            SIGUSR1 => {
                // print connections stats to the log
                let _ = tx.send(Sig::Log);
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
                    break;
                }
            }
            _ = rx.changed() => {
                let cfg = rx.borrow();
                if cfg.exit {
                    break;
                }
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
    let mut config = Cli::parse_args();

    // register signal handlers
    let signals = Signals::new(&[SIGTERM, SIGHUP, SIGUSR1])?;
    let handle = signals.handle();
    let (sig_tx, mut sig_rx) = mpsc::unbounded_channel();
    let signals_task = tokio::spawn(handle_signals(signals, sig_tx));

    let mut listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;
    // config channel, RW Lock
    let (cfg_tx, _) = watch::channel(config.clone());

    // statistics
    let (stat_tx, stat_rx) = mpsc::unbounded_channel();
    let stat_handle = tokio::spawn(background_statistic(stat_rx));

    info!("Rendlessh is running, pid {}", std::process::id());
    loop {
        tokio::select! {
            res = listener.accept() => {
                if let Ok((stream, addr)) = res {
                    let c = Client::new(addr, Instant::now(), stream, stat_tx.clone());
                    println!("accept from {}", addr);
                    stat_tx.send(StatisticEvent::NewConn).unwrap();
                    tokio::spawn(honeypot(c, cfg_tx.subscribe()));
                }
            }
            Some(sig) = sig_rx.recv() => {
                match sig {
                    Sig::Exit => break,
                    Sig::Reload => {
                        let port = config.port;
                        reload_config(&mut config);
                        if port != config.port {
                            listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;
                        }
                        cfg_tx.send(config.clone()).unwrap();
                    },
                    Sig::Log => stat_tx.send(StatisticEvent::Log).unwrap(),
                }
            }
        }
    }
    drop(stat_tx);

    // inform all clients to close
    config.exit = true;
    // if rendlessh start and immediately receive SIGTERM, no cfg_rx exist
    // and this send will fail, just ignore it instead of calling `unwrap()`
    let _ = cfg_tx.send(config);

    handle.close();
    let _ = signals_task.await;

    // stat_handle will wait all clients to drop stat_tx, so it's sufficient to gurantee
    // all clients exit and generate their statistics
    let _ = stat_handle.await;
    info!("Gracefully exit");
    Ok(())
}
