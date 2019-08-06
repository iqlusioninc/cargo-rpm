//! `cargo rpm` subcommands

pub mod build;
pub mod init;
pub mod version;

use self::{build::BuildCmd, init::InitCmd, version::VersionCmd};
use crate::config::{CargoConfig, CARGO_CONFIG_FILE};
use abscissa_core::{Command, Configurable, FrameworkError, Help, Options, Runnable};
use std::path::PathBuf;

/// `cargo` subcommand
#[derive(Command, Debug, Options, Runnable)]
pub enum CargoRpmCmd {
    /// The `cargo rpm` subcommand
    #[options(help = "build RPMs from Rust projects using Cargo")]
    Rpm(RpmCommand),
}

/// `cargo rpm` subcommands
#[derive(Command, Debug, Options, Runnable)]
pub enum RpmCommand {
    /// The `cargo rpm help` subcommand
    #[options(help = "get usage information")]
    Help(Help<Self>),

    /// The `cargo rpm build` subcommand
    #[options(help = "build an RPM out of the current project")]
    Build(BuildCmd),

    /// The `cargo rpm init` subcommand
    #[options(help = "initialize a Rust project with RPM support")]
    Init(InitCmd),

    /// The `cargo rpm version` subcommand
    #[options(help = "display version information")]
    Version(VersionCmd),
}

impl Configurable<CargoConfig> for CargoRpmCmd {
    /// Location of `Cargo.toml`
    fn config_path(&self) -> Option<PathBuf> {
        // TODO(tarcieri): recurse through parent directories (see #27)
        let filename = PathBuf::from(CARGO_CONFIG_FILE);

        if filename.exists() {
            Some(filename)
        } else {
            None
        }
    }

    /// Override `Config.toml` config values using command-line arguments
    fn process_config(&self, config: CargoConfig) -> Result<CargoConfig, FrameworkError> {
        match self {
            // TODO(tarcieri): actually use this
            CargoRpmCmd::Rpm(_cmd) => Ok(config),
        }
    }
}
