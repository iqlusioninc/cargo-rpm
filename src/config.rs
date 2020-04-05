//! `Cargo.toml` parser specialized for the `cargo rpm` use case

use crate::error::Error;
use abscissa_core::Config;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process,
};

/// Cargo configuration for the current project
pub const CARGO_CONFIG_FILE: &str = "Cargo.toml";

/// The parts of `Cargo.toml` that `cargo rpm` cares about
#[derive(Config, Debug, Default, Deserialize)]
pub struct CargoConfig {
    /// Cargo package configuration
    package: Option<PackageConfig>,
}

impl CargoConfig {
    /// The `[package]` section of `Cargo.toml`
    pub fn package(&self) -> &PackageConfig {
        self.package.as_ref().unwrap_or_else(|| {
            status_err!("no [package] section in Cargo.toml!");
            process::exit(1);
        })
    }
}

/// Struct representing possible license formats for Cargo.toml
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CargoLicense {
    /// SPDX identifier.
    License(String),

    /// Filename.
    LicenseFile(String),
}

/// Struct representing a `Cargo.toml` file's `[package]` section
#[derive(Clone, Debug, Deserialize)]
pub struct PackageConfig {
    /// Name of the package
    pub name: String,

    /// Description of the package
    pub description: String,

    /// Version of the package
    pub version: String,

    /// License of the package
    #[serde(flatten)]
    pub license: CargoLicense,

    /// Homepage of the package
    pub homepage: Option<String>,

    /// Package metadata table
    pub metadata: Option<PackageMetadata>,
}

impl PackageConfig {
    /// Get the RpmConfig for this package (if present)
    pub fn rpm_metadata(&self) -> Option<&RpmConfig> {
        self.metadata.as_ref().and_then(|md| md.rpm.as_ref())
    }

    /// Get the version and release for this package
    pub fn version(&self) -> (String, String) {
        let version_split: Vec<&str> = self.version.split('-').collect();
        let version = version_split[0].into();
        // Get the release, defaulting to 1 if there isn't one present
        // For a pre-release version, cargo release appends -alpha.0, -beta.1, etc.
        let release = version_split
            .get(1)
            .map(|rel| format!("0.{}", rel))
            .unwrap_or_else(|| "1".into());
        (version, release)
    }
}

/// The `[package.metadata]` table: ignored by Cargo, but we can put stuff there
#[derive(Clone, Debug, Deserialize)]
pub struct PackageMetadata {
    /// Our custom RPM metadata extension to `Cargo.toml`
    pub rpm: Option<RpmConfig>,
}

/// Our `[package.metadata.rpm]` extension to `Cargo.toml`
#[derive(Clone, Debug, Deserialize)]
pub struct RpmConfig {
    /// Options for creating the release artifact
    pub cargo: Option<CargoFlags>,

    /// Target configuration: a map of target binaries to their file config
    pub targets: BTreeMap<String, FileConfig>,

    /// Extra files (taken from the `.rpm` directory) to include in the RPM
    pub files: Option<BTreeMap<String, FileConfig>>,

    /// Target architecture passed to `rpmbuild`
    pub target_architecture: Option<String>,
}

/// Options for creating the release artifact
#[derive(Clone, Debug, Deserialize)]
pub struct CargoFlags {
    /// Release profile to use (default "release")
    pub profile: Option<String>,

    /// The target - defaults to the host architecture
    pub target: Option<String>,

    /// Flags to pass to cargo build
    pub buildflags: Option<Vec<String>>,
}

/// Properties of a file to be included in the final RPM
#[derive(Clone, Debug, Deserialize)]
pub struct FileConfig {
    /// Absolute path where the file should reside after installation
    pub path: PathBuf,

    /// Username of the owner of the file
    pub username: Option<String>,

    /// Groupname of the owner of the file
    pub groupname: Option<String>,

    /// Mode of the file (default 755 for targets, 644 for extra files)
    pub mode: Option<String>,
}

/// Render `package.metadata.rpm` section to include in Cargo.toml
pub fn append_rpm_metadata(
    path: &Path,
    targets: &[String],
    extra_files: &[PathBuf],
    bin_dir: &Path,
) -> Result<(), Error> {
    assert!(!targets.is_empty(), "no target configuration?!");

    status_ok!("Updating", "{}", path.canonicalize().unwrap().display());

    let mut cargo_toml = OpenOptions::new().append(true).open(path)?;

    // Flags to pass to cargo when doing a release
    // TODO: use serde serializer?
    writeln!(cargo_toml, "\n[package.metadata.rpm.cargo]")?;
    writeln!(cargo_toml, "buildflags = [\"--release\"]")?;

    // Target files to include in an archive
    writeln!(cargo_toml, "\n[package.metadata.rpm.targets]")?;

    for target in targets {
        writeln!(
            cargo_toml,
            "{} = {{ path = {:?} }}",
            target,
            bin_dir.join(target)
        )?;
    }

    // These files come from the .rpm directory
    if !extra_files.is_empty() {
        writeln!(cargo_toml, "\n[package.metadata.rpm.files]")?;

        for path in extra_files {
            if !path.is_absolute() {
                status_err!("path is not absolute: {}", path.display());
                process::exit(1);
            }

            let file = path.file_name().unwrap_or_else(|| {
                status_err!("path has no filename: {}", path.display());
                process::exit(1);
            });

            writeln!(
                cargo_toml,
                "{:?} = {{ path = {:?} }}",
                file.to_str().unwrap(),
                path.display()
            )?;
        }
    }

    Ok(())
}
