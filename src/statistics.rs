use std::{
    fmt::{self, Display},
    time::Instant,
};

pub struct Statistics {
    start: Instant,
    pub current_connects: u64,
    pub total_connects: u64,
    pub total_bytes_sent: usize,
}

impl Statistics {
    pub fn new() -> Self {
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
