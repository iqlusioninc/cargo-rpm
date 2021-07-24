//! The `cargo rpm build` subcommand

use crate::{
    builder::{Builder, RPM_CONFIG_DIR},
    prelude::*,
    target,
};
use abscissa_core::{Command, Runnable};
use gumdrop::Options;
use std::{env, path::PathBuf, process};

/// The `cargo rpm build` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct BuildCmd {
    /// Print additional information about the build
    #[options(long = "verbose")]
    pub verbose: bool,

    /// Assume that the project is already built (disables automatic cargo build)
    #[options(long = "no-cargo-build")]
    pub no_cargo_build: bool,

    /// Rust target for cross-compilation
    #[options(long = "target")]
    pub target: Option<String>,

    /// Location to the rpm config directory
    #[options(long = "config")]
    pub config: Option<String>,

    /// Output path for the built rpm (either a file or directory)
    #[options(long = "output")]
    pub output: Option<String>,
}

impl Runnable for BuildCmd {
    /// Invoke the `cargo rpm build` subcommand
    fn run(&self) {
        // Calculate paths relative to the current directory
        let crate_root = PathBuf::from(".");
        let mut rpm_config_dir = crate_root.join(RPM_CONFIG_DIR);

        // Read Cargo.toml
        let config = app_config();
        let config = config.package();
        let target_dir = target::find_dir().unwrap_or_else(|e| {
            status_err!("error finding target directory: {}", e);
            process::exit(1);
        });

        let mut output_path = None;

        // Set config and output directories from Cargo.toml
        if let Some(config_dir) = config.rpm_metadata().and_then(|rpm| rpm.config.as_ref()) {
            rpm_config_dir = crate_root.join(config_dir);
        }
        if let Some(output_dir) = config.rpm_metadata().and_then(|rpm| rpm.output.as_ref()) {
            output_path = Some(crate_root.join(output_dir).display().to_string());
        }

        // Set config directory from argument and convert it to an absolute path
        if let Some(config_path) = &self.config {
            let current_dir = env::current_dir().unwrap_or_else(|err| {
                status_err!("{}", err);
                process::exit(1);
            });

            // Similar logic as below for output path
            rpm_config_dir = current_dir.join(config_path);
        }

        // Convert the specified output path string to an absolute path. This
        // ensures that when relative paths are specified as cargo rpm output,
        // rpmbuild respects it (this path ultimately gets passed to rpmbuild
        // and if we don't do this, rpmbuild would put the rpm relative to
        // %{_topdir}, when relative paths are specified here).
        let convert_to_absolute = |path_string| {
            let mut absolute = env::current_dir().unwrap_or_else(|err| {
                status_err!("{}", err);
                process::exit(1);
            });
            // If `path_string` is already absolute, `absolute` becomes that. Otherwise
            // current dir is prepended to the `path_string`.
            absolute.push(path_string);
            absolute.display().to_string()
        };

        // Set the output path from argument or Cargo.toml
        if self.output.is_some() {
            output_path = self.output.as_ref().map(convert_to_absolute);
        } else {
            output_path = output_path.as_ref().map(convert_to_absolute);
        }

        Builder::new(
            config,
            self.verbose,
            self.no_cargo_build,
            self.target.as_ref(),
            output_path.as_ref(),
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
