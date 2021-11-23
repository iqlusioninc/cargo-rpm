//! Target-type autodetection for crates

use cargo_metadata::MetadataCommand;

use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use std::{
    env,
    path::{Path, PathBuf},
};

/// Locate the project's target directory
pub fn find_dir() -> Result<PathBuf, Error> {
    // Allow for an explicit override of the target directory.
    if let Some(p) = env::var_os("CARGO_TARGET_DIR") {
        return Ok(PathBuf::from(p));
    }

    cargo_metadata::MetadataCommand::new()
        .exec()
        .map(|metadata| metadata.target_directory.into())
        .map_err(|err| format_err!(ErrorKind::Target, "failed to fetch metadata: {}", err).into())
}

/// Targets we can autodetect
#[derive(PartialEq, Eq)]
pub enum Target {
    /// A shared object library
    Cdylib(String),

    /// A binary executable
    Bin(String),
}

impl Target {
    /// Autodetect the targets for this crate
    pub fn detect(base_path: &Path) -> Result<Vec<Self>, Error> {
        let metadata = MetadataCommand::new()
            .current_dir(base_path)
            .no_deps()
            .exec()
            .unwrap();
        Ok(metadata.packages[0]
            .targets
            .iter()
            .filter_map(|target| {
                if target.kind.iter().any(|t| t == "bin") {
                    Some(Self::Bin(target.name.to_string()))
                } else if target.kind.iter().any(|t| t == "cdylib") {
                    Some(Self::Cdylib(target.name.to_string()))
                } else {
                    None
                }
            })
            .collect())
    }
}
