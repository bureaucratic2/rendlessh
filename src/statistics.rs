use log::info;
use std::{
    fmt::{self, Display},
    time::Instant,
};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub enum StatisticEvent {
    NewConn,
    BytesSent(usize),
    DropConn,
    Log,
}

struct Statistics {
    start: Instant,
    pub current_connects: u64,
    pub total_connects: u64,
    pub total_bytes_sent: usize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            current_connects: 0,
            total_connects: 0,
            total_bytes_sent: 0,
        }
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Total connects={}, current connects={}, last {}s, {} bytes sent",
            self.total_connects,
            self.current_connects,
            self.start.elapsed().as_millis() as f64 / 1000.0,
            self.total_bytes_sent
        )
    }
}

pub async fn background_statistic(mut stat_rx: UnboundedReceiver<StatisticEvent>) {
    let mut stat = Statistics::new();
    while let Some(event) = stat_rx.recv().await {
        match event {
            StatisticEvent::NewConn => {
                stat.current_connects += 1;
                stat.total_connects += 1;
            }
            StatisticEvent::BytesSent(n) => stat.total_bytes_sent += n,
            StatisticEvent::DropConn => stat.current_connects -= 1,
            StatisticEvent::Log => info!("{}", &stat),
        }
    }
    info!("gracefully exit, generate statistic information");
    info!("{}", &stat);
}
