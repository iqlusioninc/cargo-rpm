//! RPM builder

use crate::{
    archive::Archive,
    config::{PackageConfig, RpmConfig},
    error::Error,
    rpmbuild::Rpmbuild,
};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{self, Command},
    time::Instant,
};

/// Default build profile to use
pub const DEFAULT_PROFILE: &str = "release";

/// Subdirectory of a Rust project in which we keep RPM-related configs
pub const RPM_CONFIG_DIR: &str = ".rpm";

/// Placeholder string in the `.spec` file we use for the version
pub const VERSION_PLACEHOLDER: &str = "@@VERSION@@";

/// Placeholder string in the `.spec` file we use for the release
pub const RELEASE_PLACEHOLDER: &str = "@@RELEASE@@";

/// Build RPMs from Rust projects
pub struct Builder {
    /// Cargo.toml configuration
    pub config: PackageConfig,

    /// Are we in verbose mode?
    pub verbose: bool,

    /// RPM configuration directory (i.e. `.rpm`)
    pub rpm_config_dir: PathBuf,

    /// Path to the target directory
    pub target_dir: PathBuf,

    /// Path to the rpmbuild directory
    pub rpmbuild_dir: PathBuf,
}

impl Builder {
    /// Create a new RPM builder
    pub fn new(
        config: &PackageConfig,
        verbose: bool,
        rpm_config_dir: &Path,
        base_target_dir: &Path,
    ) -> Self {
        let mut profile = DEFAULT_PROFILE.to_owned();
        // Default target is empty.
        let mut target = "".to_owned();
        {
            let rpm_metadata = config.rpm_metadata().unwrap_or_else(|| {
                status_err!("No [package.metadata.rpm] in Cargo.toml!");
                println!("\nRun 'cargo rpm init' to configure crate for RPM builds");

                process::exit(1);
            });

            if let Some(ref cargo) = rpm_metadata.cargo {
                if let Some(ref p) = cargo.profile {
                    profile = p.to_owned();
                }
                if let Some(ref t) = cargo.target {
                    target = t.to_owned();
                }
            }
        }

        let target_dir = base_target_dir.join(target).join(profile);
        let rpmbuild_dir = target_dir.join("rpmbuild");

        Self {
            config: config.clone(),
            verbose,
            rpm_config_dir: rpm_config_dir.into(),
            target_dir,
            rpmbuild_dir,
        }
    }

    /// Build an RPM for this package
    pub fn build(&self) -> Result<(), Error> {
        let began_at = Instant::now();

        self.cargo_build()?;
        self.create_archive()?;
        self.render_spec()?;
        self.rpmbuild()?;

        let (version, release) = self.config.version();

        status_ok!(
            "Finished",
            "{}-{}-{}.rpm: built in {} secs",
            self.config.rpm_name(),
            version,
            release,
            began_at.elapsed().as_secs()
        );

        Ok(())
    }

    /// Retrieve the RPM metadata for this crate
    fn rpm_metadata(&self) -> &RpmConfig {
        self.config.rpm_metadata().unwrap()
    }

    /// Compile the project with "cargo build"
    fn cargo_build(&self) -> Result<(), Error> {
        let mut buildflags = vec![];

        if let Some(ref cargo) = self.rpm_metadata().cargo {
            if let Some(ref t) = cargo.target {
                buildflags.push(format!("--target={}", t));
            }

            if let Some(ref b) = cargo.buildflags {
                buildflags.append(&mut b.clone());
            }
        };

        if self.verbose {
            status_ok!("Running", "cargo build {}", buildflags.join(" "));
        }

        let status = Command::new("cargo")
            .arg("build")
            .args(&buildflags)
            .status()?;

        // Exit with the same exit code cargo used
        if !status.success() {
            process::exit(status.code().unwrap_or(1));
        }

        Ok(())
    }

    /// Create the archive (i.e. tarball) containing targets and additional files
    fn create_archive(&self) -> Result<(), Error> {
        let sources_dir = self.rpmbuild_dir.join("SOURCES");
        fs::create_dir_all(&sources_dir)?;

        let (version, _) = self.config.version();

        // Build a tarball containing the RPM's contents
        let archive_file = format!("{}-{}.tar.gz", self.config.rpm_name(), version);
        let archive_path = sources_dir.join(&archive_file);

        if self.verbose {
            status_ok!("Creating", "release archive: {}", &archive_file);
        }

        Archive::new(&self.config, &self.rpm_config_dir, &self.target_dir)?.build(&archive_path)?;

        Ok(())
    }

    /// Render the package's RPM spec file
    fn render_spec(&self) -> Result<(), Error> {
        // Read the spec file from `.rpm`
        let spec_filename = format!("{}.spec", self.config.rpm_name());
        let mut spec_src = File::open(self.rpm_config_dir.join(&spec_filename))?;
        let mut spec_template = String::new();
        spec_src.read_to_string(&mut spec_template)?;

        let (version, release) = self.config.version();

        // Replace `@@VERSION@@` with the crate's actual version
        let spec_ver_rendered = str::replace(&spec_template, VERSION_PLACEHOLDER, &version);

        // Replace `@@RELEASE@@` with the crate's release
        let spec_rendered = str::replace(&spec_ver_rendered, RELEASE_PLACEHOLDER, &release);

        let spec_dir = self.rpmbuild_dir.join("SPECS");
        fs::create_dir_all(&spec_dir)?;

        let mut spec_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(spec_dir.join(&spec_filename))?;

        spec_file.write_all(spec_rendered.as_bytes())?;

        Ok(())
    }

    /// Run rpmbuild
    fn rpmbuild(&self) -> Result<(), Error> {
        let (version, release) = self.config.version();
        let rpm_file = format!("{}-{}-{}.rpm", self.config.rpm_name(), version, release);
        let cmd = Rpmbuild::new(self.verbose)?;

        status_ok!(
            "Building",
            "{} (using rpmbuild {})",
            rpm_file,
            cmd.version().unwrap()
        );

        // Create directories needed by rpmbuild
        for dir in &["RPMS", "SRPMS", "BUILD", "SOURCES", "SPECS", "tmp"] {
            fs::create_dir_all(self.rpmbuild_dir.join(dir))?;
        }

        // Change directory to `target/<profile>/rpmbuild`
        env::set_current_dir(&self.rpmbuild_dir)?;

        // Calculate rpmbuild arguments
        let spec_path = format!("SPECS/{}.spec", self.config.rpm_name());
        let topdir_macro = format!("_topdir {}", self.rpmbuild_dir.display());
        let tmppath_macro = format!("_tmppath {}", self.rpmbuild_dir.join("tmp").display());

        // Calculate rpmbuild arguments
        let args = ["-D", &topdir_macro, "-D", &tmppath_macro, "-ba", &spec_path];

        if self.verbose {
            status_ok!("Running", "{} {}", cmd.path.display(), &args.join(" "));
        }

        // Actually run rpmbuild
        cmd.exec(&args)
    }
}
