//! Wrapper for running the `rpmbuild` command

use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use std::{
    ffi::OsStr,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
};

/// Path to the `rpmbuild` command
pub const DEFAULT_RPMBUILD_PATH: &str = "rpmbuild";

/// Version of rpmbuild supported by this tool
pub const SUPPORTED_RPMBUILD_VERSION: &str = " 4.";

/// Wrapper for the `rpmbuild` command
pub struct Rpmbuild {
    /// Path to rpmbuild
    pub path: PathBuf,

    /// Are we in verbose mode?
    pub verbose: bool,
}

impl Rpmbuild {
    /// Prepare `rpmbuild`, checking the correct version is installed
    pub fn new(verbose: bool) -> Result<Self, Error> {
        let rpmbuild = Self {
            path: DEFAULT_RPMBUILD_PATH.into(),
            verbose,
        };

        // Make sure we have a valid version of rpmbuild
        rpmbuild.version()?;
        Ok(rpmbuild)
    }

    /// Get version of `rpmbuild`
    pub fn version(&self) -> Result<String, Error> {
        let output = Command::new(&self.path)
            .args(&["--version"])
            .output()
            .map_err(|e| {
                format_err!(
                    ErrorKind::Rpmbuild,
                    "error running {}: {}",
                    self.path.display(),
                    e
                )
            })?;

        if !output.status.success() {
            fail!(
                ErrorKind::Rpmbuild,
                "error running {} (exit status: {})",
                &self.path.display(),
                &output.status
            );
        }

        let vers = String::from_utf8(output.stdout).map_err(|e| {
            format_err!(
                ErrorKind::Rpmbuild,
                "error parsing rpmbuild output as UTF-8: {}",
                e
            )
        })?;

        if !vers.contains(SUPPORTED_RPMBUILD_VERSION) {
            fail!(
                ErrorKind::Rpmbuild,
                "unexpected rpmbuild version string: {:?}",
                vers
            );
        }

        let parts: Vec<&str> = vers.split_whitespace().collect();
        Ok(parts[parts.len() - 1].to_owned())
    }

    /// Execute `rpmbuild` with the given arguments
    pub fn exec<I, S>(&self, args: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut rpmbuild = Command::new(&self.path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(if self.verbose {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .spawn()
            .map_err(|e| {
                format_err!(
                    ErrorKind::Rpmbuild,
                    "error running {}: {}",
                    self.path.display(),
                    e
                )
            })?;

        let output = self.read_rpmbuild_output(&mut rpmbuild)?;
        let status = rpmbuild.wait()?;

        if status.success() {
            Ok(())
        } else {
            if !self.verbose {
                eprintln!("{}", &output);
            }

            fail!(
                ErrorKind::Rpmbuild,
                "error running {} (exit status: {})",
                self.path.display(),
                status
            );
        }
    }

    /// Read stdout from rpmbuild, either displaying it or discarding it
    fn read_rpmbuild_output(&self, subprocess: &mut Child) -> Result<String, Error> {
        let mut reader = BufReader::new(subprocess.stdout.as_mut().unwrap());
        let mut string = String::new();

        while reader.read_line(&mut string)? != 0 {
            if self.verbose {
                status_ok!("rpmbuild", string.trim_end());
                string.clear();
            }
        }

        Ok(string)
    }
}
