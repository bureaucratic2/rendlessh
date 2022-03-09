use std::fmt::{self, Display};
use std::path::PathBuf;

pub use crate::cli::{reload_config, Cli};
pub use crate::client::Client;
pub use crate::error::Result;
pub use crate::statistics::{background_statistic, StatisticEvent};

use crate::logger::setup_logging;

const DEFAULT_PORT: u32 = 2222;
const DEFAULT_DELAY: u64 = 10000;
const DEFAULT_MAX_LEN: usize = 32;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u32,
    pub delay: u64,
    pub length: usize,
    path: PathBuf,

    pub exit: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            delay: DEFAULT_DELAY,
            length: DEFAULT_MAX_LEN,
            path: PathBuf::default(),

            exit: false,
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

mod cli;
mod client;
mod error;
mod logger;
mod statistics;
