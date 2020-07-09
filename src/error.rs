//! Error types

use crate::prelude::*;
use abscissa_core::error::{BoxError, Context};
use std::{fmt, io, num, ops::Deref, path, time};
use thiserror::Error;

/// Kinds of errors
#[derive(Copy, Clone, Eq, Error, PartialEq, Debug)]
pub enum ErrorKind {
    /// Error in `Cargo.toml`
    #[error("config error")]
    Config,

    /// Errors related to dates/times
    #[error("date/time error")]
    Date,

    /// License type errors
    #[error("license error")]
    License,

    /// Input/output error
    #[error("I/O error")]
    Io,

    /// Errors parsing data
    #[error("parse error")]
    Parse,

    /// Errors relating to file paths
    #[error("path error")]
    Path,

    /// Errors invoking the `rpmbuild` utility
    #[error("rpmbuild error")]
    Rpmbuild,

    /// Errors involving the target binary
    #[error("target error")]
    Target,

    /// Errors related to template files (for RPM specs, systemd service units, etc)
    #[error("template error")]
    Template,
}

impl ErrorKind {
    /// Create an error context from this error
    pub fn context(self, source: impl Into<BoxError>) -> Context<ErrorKind> {
        Context::new(self, Some(source.into()))
    }
}

/// Error type
#[derive(Debug)]
pub struct Error(Box<Context<ErrorKind>>);

impl Deref for Error {
    type Target = Context<ErrorKind>;

    fn deref(&self) -> &Context<ErrorKind> {
        &self.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Context::new(kind, None).into()
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(context: Context<ErrorKind>) -> Self {
        Error(Box::new(context))
    }
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        format_err!(ErrorKind::Io, other).into()
    }
}

impl From<num::ParseIntError> for Error {
    fn from(other: num::ParseIntError) -> Self {
        format_err!(ErrorKind::Parse, other).into()
    }
}

impl From<path::StripPrefixError> for Error {
    fn from(other: path::StripPrefixError) -> Self {
        format_err!(ErrorKind::Path, other).into()
    }
}

impl From<time::SystemTimeError> for Error {
    fn from(other: time::SystemTimeError) -> Self {
        format_err!(ErrorKind::Date, other).into()
    }
}
