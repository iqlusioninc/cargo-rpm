//! The `cargo rpm build` subcommand

use crate::{
    builder::{Builder, RPM_CONFIG_DIR},
    prelude::*,
    target,
};
use abscissa_core::{Command, Runnable};
use std::{path::PathBuf, process};

/// The `cargo rpm build` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct BuildCmd {
    /// Print additional information about the build
    #[options(long = "verbose")]
    pub verbose: bool,

    /// Assume that the project is already built (disables automatic cargo build)
    #[options(long = "no-cargo-build")]
    pub no_cargo_build: bool,
}

impl Runnable for BuildCmd {
    /// Invoke the `cargo rpm build` subcommand
    fn run(&self) {
        // Calculate paths relative to the current directory
        let crate_root = PathBuf::from(".");
        let rpm_config_dir = crate_root.join(RPM_CONFIG_DIR);

        // Read Cargo.toml
        let config = app_config();
        let target_dir = target::find_dir().unwrap_or_else(|e| {
            status_err!("error finding target directory: {}", e);
            process::exit(1);
        });

        Builder::new(
            config.package(),
            self.verbose,
            self.no_cargo_build,
            &rpm_config_dir,
            &target_dir,
        )
        .build()
        .unwrap_or_else(|err| {
            status_err!("{}", err);
            process::exit(1);
        })
    }
}
