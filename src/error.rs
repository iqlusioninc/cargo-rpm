//! Error types

use abscissa_core::err;
use failure::Fail;
use std::{fmt, io, num, path, time};

/// Error type
#[derive(Debug)]
pub struct Error(abscissa_core::Error<ErrorKind>);

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    /// Error in `Cargo.toml`
    #[fail(display = "config error")]
    Config,

    /// Errors related to dates/times
    #[fail(display = "date/time error")]
    Date,

    /// License type errors
    #[fail(display = "license error")]
    License,

    /// Input/output error
    #[fail(display = "I/O error")]
    Io,

    /// Errors parsing data
    #[fail(display = "parse error")]
    Parse,

    /// Errors relating to file paths
    #[fail(display = "path error")]
    Path,

    /// Errors invoking the `rpmbuild` utility
    #[fail(display = "rpmbuild error")]
    Rpmbuild,

    /// Errors involving the target binary
    #[fail(display = "target error")]
    Target,

    /// Errors related to template files (for RPM specs, systemd service units, etc)
    #[fail(display = "template error")]
    Template,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<abscissa_core::Error<ErrorKind>> for Error {
    fn from(other: abscissa_core::Error<ErrorKind>) -> Self {
        Error(other)
    }
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        err!(ErrorKind::Io, other).into()
    }
}

impl From<num::ParseIntError> for Error {
    fn from(other: num::ParseIntError) -> Self {
        err!(ErrorKind::Parse, other).into()
    }
}

impl From<path::StripPrefixError> for Error {
    fn from(other: path::StripPrefixError) -> Self {
        err!(ErrorKind::Path, other).into()
    }
}

impl From<time::SystemTimeError> for Error {
    fn from(other: time::SystemTimeError) -> Self {
        err!(ErrorKind::Date, other).into()
    }
}
