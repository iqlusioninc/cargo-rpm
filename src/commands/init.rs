//! The `cargo rpm init` subcommand

use crate::{
    builder::RPM_CONFIG_DIR,
    config::{self, PackageConfig, CARGO_CONFIG_FILE},
    error::Error,
    prelude::*,
    target::TargetType,
    templates::{ServiceParams, SpecParams},
};
use abscissa_core::Command;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process,
};

/// Directory in which systemd service unit configs reside
const SYSTEMD_DIR: &str = "/usr/lib/systemd/system";

/// The `cargo rpm init` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct InitCmd {
    /// Force (re-)generation, even if .rpm exists or the target type is unsupported
    #[options(long = "force")]
    pub force: bool,

    /// Place binaries in `/usr/sbin` instead of `/usr/bin`?
    #[options(no_short, long = "sbin")]
    pub sbin: bool,

    /// Path to the systemd service unit config template
    #[options(no_short, long = "service")]
    pub service: Option<String>,

    /// Configure this RPM as a systemd service unit
    #[options(short = "s", long = "systemd")]
    pub systemd: bool,

    /// Path to the RPM spec template
    #[options(long = "template")]
    pub template: Option<String>,
}

impl Runnable for InitCmd {
    /// Invoke the `cargo rpm init` subcommand
    fn run(&self) {
        self.init().unwrap_or_else(|err| {
            status_err!("{}", err);
            process::exit(1);
        })
    }
}

impl InitCmd {
    /// Initialize the `.rpm` subdirectory
    // TODO(tarcieri): refactor this into smaller (testable) functions
    pub fn init(&self) -> Result<(), Error> {
        // Calculate paths relative to the current directory
        let crate_root = PathBuf::from(".");
        let cargo_toml = crate_root.join(CARGO_CONFIG_FILE);
        let rpm_config_dir = crate_root.join(RPM_CONFIG_DIR);

        // Read Cargo.toml
        let config = app_config();

        // Check if `.rpm` already exists
        if rpm_config_dir.exists() {
            if self.force {
                let canonical_rpm_config_dir = rpm_config_dir.canonicalize()?;
                status_warn!("deleting {} (forced)", canonical_rpm_config_dir.display());
                fs::remove_dir_all(&rpm_config_dir)?;
            } else {
                status_err!(
                    "destination `{}` already exists!",
                    rpm_config_dir.canonicalize().unwrap().display()
                );
                process::exit(1);
            }
        }

        // Check if we're creating a systemd service unit for this crate
        let service_name = if self.service.is_some() || self.systemd {
            Some(format!("{}.service", config.package().name))
        } else {
            None
        };

        // Autodetect whether to place target files in `/usr/bin` or `/usr/sbin`
        let use_sbin = self.sbin;

        // Autodetect target types
        let targets = match TargetType::detect(&crate_root)? {
            TargetType::Lib => {
                if self.force {
                    // If forced, just return an empty target list
                    vec![]
                } else {
                    status_err!("detected unsupported crate type: library (-f to override)");
                    process::exit(1);
                }
            }
            TargetType::Bin => vec![config.package().name.clone()],
            TargetType::MultiBin(targets) => targets,
        };

        // Create `.rpm` directory
        fs::create_dir(&rpm_config_dir)?;
        status_ok!(
            "Created",
            "{}",
            rpm_config_dir.canonicalize().unwrap().display()
        );

        // Render `.rpm/<cratename>.spec`
        let spec_path = rpm_config_dir.join(format!("{}.spec", config.package().name));
        let spec_params = SpecParams::new(&config.package(), service_name.clone(), use_sbin);
        render_spec(&spec_path, &self.template, &spec_params)?;

        // (Optional) Render `.rpm/<cratename>.service` (systemd service unit config)
        if let Some(ref service) = service_name {
            render_service(
                &rpm_config_dir.join(service),
                &self.service,
                config.package(),
            )?;
        }

        // Update Cargo.toml with RPM metadata
        if config.package().rpm_metadata().is_some() && !self.force {
            status_warn!("not updating Cargo.toml because [package.metadata.rpm] already present");
        } else {
            let mut extra_files = vec![];
            if let Some(ref service) = service_name {
                extra_files.push(PathBuf::from(SYSTEMD_DIR).join(service.clone()));
            }

            let bin_dir: PathBuf = if use_sbin { "/usr/sbin" } else { "/usr/bin" }.into();
            config::append_rpm_metadata(&cargo_toml, &targets, &extra_files, &bin_dir)?;
        }

        status_ok!(
            "Finished",
            "{} configured (type \"cargo rpm build\" to build)",
            config.package().name
        );

        Ok(())
    }
}

/// Render this package's RPM spec
fn render_spec(
    spec_path: &Path,
    template_path_str: &Option<String>,
    spec_params: &SpecParams,
) -> Result<(), Error> {
    let template_path = template_path_str.as_ref().map(PathBuf::from);
    let spec_rendered = spec_params.render(template_path.as_ref().map(|t| t.as_ref()))?;

    let mut spec_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(spec_path)?;

    spec_file.write_all(spec_rendered.as_bytes())?;

    status_ok!(
        "Rendered",
        "{}",
        spec_path.canonicalize().unwrap().display()
    );

    Ok(())
}

/// Render this package's systemd service unit config (if enabled)
fn render_service(
    service_path: &Path,
    template_path_str: &Option<String>,
    package_config: &PackageConfig,
) -> Result<(), Error> {
    let service_params = ServiceParams::from(package_config);
    let template_path = template_path_str.as_ref().map(PathBuf::from);
    let service_rendered = service_params.render(template_path.as_ref().map(|t| t.as_ref()))?;

    let mut service_file = File::create(service_path)?;
    service_file.write_all(service_rendered.as_bytes())?;

    status_ok!(
        "Rendered",
        "{}",
        service_path.canonicalize().unwrap().display()
    );

    Ok(())
}
