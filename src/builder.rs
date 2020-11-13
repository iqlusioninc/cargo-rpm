//! RPM builder

use crate::{
    archive::Archive,
    config::{PackageConfig, RpmConfig},
    error::Error,
    prelude::*,
    rpmbuild::Rpmbuild,
    target_architecture::TargetArch,
};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
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

    /// Can we assume that the project is already built?
    pub no_cargo_build: bool,

    /// Rust target for cross-compilation
    pub target: Option<String>,

    /// Output path for the built rpm (either a file or directory)
    pub output_path: Option<String>,

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
        no_cargo_build: bool,
        target: Option<&String>,
        output_path: Option<&String>,
        rpm_config_dir: &Path,
        base_target_dir: &Path,
    ) -> Self {
        let mut profile = DEFAULT_PROFILE.to_owned();
        let mut config_target = None;

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
                config_target = cargo.target.as_ref();
            }
        }

        if target.is_some() && config_target.is_some() {
            status_warn!("target also specified as part of [package.metadata.rpm.cargo] in Cargo.toml, but ignoring it");
        }
        let final_target = target.or(config_target);

        let target_dir = base_target_dir
            .join(final_target.unwrap_or(&"".to_owned())) // empty default target
            .join(profile);
        let rpmbuild_dir = target_dir.join("rpmbuild");

        Self {
            config: config.clone(),
            verbose,
            no_cargo_build,
            target: final_target.cloned(),
            output_path: output_path.cloned(),
            rpm_config_dir: rpm_config_dir.into(),
            target_dir,
            rpmbuild_dir,
        }
    }

    /// Build an RPM for this package
    pub fn build(&self) -> Result<(), Error> {
        let began_at = Instant::now();

        if !self.no_cargo_build {
            self.cargo_build()?;
        }
        self.build_hooks()?;
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

        if let Some(ref t) = self.target {
            buildflags.push(format!("--target={}", t));
        }

        if let Some(ref cargo) = self.rpm_metadata().cargo {
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

    /// Launch commands after `cargo build`  
    fn build_hooks(&self) -> Result<(), Error> {
        for hooks in &self.rpm_metadata().build_hooks {
            for (cmd, args) in hooks {
                status_info!("Launching", "build hook \"{}\"", cmd);

                let status = Command::new(cmd)
                    .args(args)
                    .stdin(Stdio::null())
                    .stdout(if self.verbose {
                        Stdio::inherit()
                    } else {
                        Stdio::null()
                    })
                    .stderr(if self.verbose {
                        Stdio::inherit()
                    } else {
                        Stdio::null()
                    })
                    .status()?;

                if !status.success() {
                    status_err!(
                        "Failed to launch build hook \"{}\" `{}`",
                        cmd,
                        args.join(" ")
                    );
                    process::exit(status.code().unwrap_or(1));
                }
            }
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

    /// Interpret the output path string as rpm (dir, filename) pair, when it's present
    fn get_rpm_dir_and_filename(&self) -> Option<(&str, &str)> {
        self.output_path.as_ref().map(|path_string| {
            let path_str = path_string.as_str();

            if path_str.ends_with('/') || Path::new(path_str).is_dir() {
                // filename as a rpm macro string, for use by rpmbuild.
                // based on default %{_build_name_fmt} (stripping off the %{ARCH}/ subfolder)
                (path_str, "%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}.rpm")
            } else {
                let path_str_parts: Vec<&str> = path_str.rsplitn(2, '/').collect();

                let filename = path_str_parts[0];
                let dir = if path_str_parts.len() == 1 {
                    "." // current dir. example path_str: packagename.rpm
                } else if path_str_parts[1].is_empty() {
                    "/" // root dir. example path_str: /packagename.rpm
                } else {
                    path_str_parts[1]
                };

                (dir, filename)
            }
        })
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
        let mut args = vec!["-ba", &spec_path, "-D", &topdir_macro, "-D", &tmppath_macro];

        // By default, final rpm output path is:
        // %{_topdir}/RPMS/%{ARCH}/%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}.rpm
        // Change it when the output path is specified.
        let mut rpmdir_macro = "_rpmdir ".to_owned();
        let mut build_name_fmt_macro = "_build_name_fmt ".to_owned();
        if let Some((dir, filename)) = self.get_rpm_dir_and_filename() {
            rpmdir_macro.push_str(dir);
            build_name_fmt_macro.push_str(filename);
            args.extend(&["-D", &rpmdir_macro, "-D", &build_name_fmt_macro]);
        }

        // Set the rpm target architecture
        let mut arch = "".to_owned();
        if let Some(config_arch) = self
            .config
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.rpm.as_ref())
            .and_then(|rpm| rpm.target_architecture.as_ref())
        {
            arch = config_arch.to_owned();
            if self.verbose {
                status_ok!(
                    "Configuring",
                    "rpm target architecture (based on [package.metadata.rpm] from Cargo.toml): {}",
                    arch
                );
            }
        } else if let Some(target) = self.target.as_ref() {
            arch = TargetArch::parse(target)?
                .as_rpm_target_architecture()
                .to_owned();
            if self.verbose {
                status_ok!(
                    "Configuring",
                    "rpm target architecture (based on specified rust target): {}",
                    arch
                );
            }
        };
        if !arch.is_empty() {
            args.extend(&["--target", &arch]);
        }

        if self.verbose {
            status_ok!("Running", "{} {}", cmd.path.display(), &args.join(" "));
        }

        // Actually run rpmbuild
        cmd.exec(&args)
    }
}
