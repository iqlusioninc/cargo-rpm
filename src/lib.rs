//! cargo-rpm: Cargo subcommand for creating RPM releases of Rust projects

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/cargo-rpm/0.5.0")]

#[macro_use]
extern crate abscissa_core;

pub mod application;
pub mod archive;
pub mod builder;
pub mod commands;
pub mod config;
pub mod error;
pub mod license;
mod prelude;
pub mod rpmbuild;
pub mod target;
pub mod templates;
