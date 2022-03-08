//! Self-defined failure handling

use failure::Fail;
use std::{io, result};

pub type Result<T> = result::Result<T, Error>;

/// Absorb outer crates error and self-made error
#[derive(Debug, Fail)]
pub enum Error {
    /// Error from IO
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    /// Error from toml
    #[fail(display = "{}", _0)]
    TomlParseError(#[cause] toml::de::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::TomlParseError(err)
    }
}
