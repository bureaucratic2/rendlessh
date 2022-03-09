use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;

use log::info;
use toml::Value;

use crate::{setup_logging, Config, Result};

const BANNER: &str = "     ___  _____  _____  __   ______________ __\n    / _ \\/ __/ |/ / _ \\/ /  / __/ __/ __/ // /\n   / , _/ _//    / // / /__/ _/_\\ \\_\\ \\/ _  /\n  /_/|_/___/_/|_/____/____/___/___/___/_//_/";

#[derive(Parser)]
#[clap(author, version, about=BANNER)]
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

    /// Log level (repeatable)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u64,
}

impl Cli {
    pub fn parse_args() -> Config {
        let cli = Cli::parse();

        setup_logging(cli.verbose).expect("Fail to set global logger");

        let mut config = Config::default();

        if let Some(path) = cli.config {
            match parse_toml(&path, &mut config) {
                Ok(_) => info!("Load config from {:#?}, {}", path.as_os_str(), config),
                Err(err) => info!("{}", err.to_string()),
            }
            config.path = path;
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

        info!("Current config {}", config);

        config
    }
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

pub fn reload_config(cfg: &mut Config) {
    let path = cfg.path.clone();
    match parse_toml(&path, cfg) {
        Ok(_) => info!("Reload config from {:#?}, {}", path.as_os_str(), cfg),
        Err(err) => info!("{}", err.to_string()),
    }
}
