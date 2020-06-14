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

    /// Get the RPM package name. This is either the explicitly set
    /// `package.metadata.rpm.package` variable or defaults to the
    /// `package.name` crate name variable.
    pub fn rpm_name(&self) -> &str {
        self.rpm_metadata()
            .and_then(|r| r.package.as_ref())
            .unwrap_or(&self.name)
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
    /// The RPM package name, if different from crate name
    pub package: Option<String>,

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
    pkg_name: &str,
    path: &Path,
    targets: &[String],
    extra_files: &[PathBuf],
    bin_dir: &Path,
) -> Result<(), Error> {
    assert!(!targets.is_empty(), "no target configuration?!");

    status_ok!("Updating", "{}", path.canonicalize().unwrap().display());

    let mut cargo_toml = OpenOptions::new().append(true).open(path)?;

    writeln!(cargo_toml, "\n[package.metadata.rpm]")?;
    writeln!(cargo_toml, "package = \"{}\"", pkg_name)?;

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

/// Get rpm target architecture based on the rust target
pub fn get_target_architecture(target: &str) -> &str {
    let mut parts = target.split('-');
    let arch = parts.next().unwrap();
    let abi = parts.last().unwrap_or("");

    // based on the similar function from cargo-deb:
    // https://github.com/mmstick/cargo-deb/blob/v1.23.1/src/manifest.rs#L909-L937
    // with adjustments for valid values that `rpmbuild --target` takes
    match (arch, abi) {
        // https://doc.rust-lang.org/std/env/consts/constant.ARCH.html
        // rustc --print target-list
        // https://fedoraproject.org/wiki/Architectures
        // https://github.com/rpm-software-management/rpm/blob/rpm-4.14.3-release/rpmrc.in#L156
        ("mipsisa32r6", _) => "mipsr6",
        ("mipsisa32r6el", _) => "mipsr6el",
        ("mipsisa64r6", _) => "mips64r6",
        ("mipsisa64r6el", _) => "mips64r6el",
        ("powerpc", _) => "ppc",
        ("powerpc64", _) => "ppc64",
        ("powerpc64le", _) => "ppc64le",
        ("riscv64gc", _) => "riscv64",
        ("x86", _) => "i386",
        (arm, gnueabi) if arm.starts_with("arm") && gnueabi.ends_with("hf") => "armv7hl",
        (arm, _) if arm.starts_with("arm") => "armv7l",
        (other_arch, _) => other_arch,
    }
}
