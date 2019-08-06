//! `cargo-rpm` application definition

use crate::{commands::CargoRpmCmd, config::CargoConfig};
use abscissa_core::{
    application, config, logging, Application, EntryPoint, FrameworkError, StandardPaths,
};
use lazy_static::lazy_static;

lazy_static! {
    /// Application state
    pub static ref APPLICATION: application::Lock<CargoRpmApp> = application::Lock::default();
}

/// Obtain a read-only (multi-reader) lock on the application state.
///
/// Panics if the application state has not been initialized.
pub fn app_reader() -> application::lock::Reader<CargoRpmApp> {
    APPLICATION.read()
}

/// Obtain an exclusive mutable lock on the application state.
pub fn app_writer() -> application::lock::Writer<CargoRpmApp> {
    APPLICATION.write()
}

/// Obtain a read-only (multi-reader) lock on the application configuration.
///
/// Panics if the application configuration has not been loaded.
pub fn app_config() -> config::Reader<CargoRpmApp> {
    config::Reader::new(&APPLICATION)
}

/// CargoRpm Application
#[derive(Debug)]
pub struct CargoRpmApp {
    /// Configured `[package.metadata.rpm]` from `Cargo.toml`
    config: Option<CargoConfig>,

    /// Application state.
    state: application::State<Self>,
}

impl Default for CargoRpmApp {
    fn default() -> Self {
        Self {
            config: None,
            state: application::State::default(),
        }
    }
}

impl Application for CargoRpmApp {
    /// Entrypoint command for this application.
    type Cmd = EntryPoint<CargoRpmCmd>;

    /// Current Cargo project's `[package.metadata.rpm]` settings from `Cargo.toml`
    type Cfg = CargoConfig;

    /// Paths to resources within the application.
    type Paths = StandardPaths;

    /// Accessor for `cargo rpm` metadata from `Cargo.toml`
    fn config(&self) -> &CargoConfig {
        self.config.as_ref().expect("config not loaded")
    }

    /// Borrow the application state immutably.
    fn state(&self) -> &application::State<Self> {
        &self.state
    }

    /// Borrow the application state mutably.
    fn state_mut(&mut self) -> &mut application::State<Self> {
        &mut self.state
    }

    /// Register all components used by this application.
    fn register_components(&mut self, command: &Self::Cmd) -> Result<(), FrameworkError> {
        let components = self.framework_components(command)?;
        self.state.components.register(components)
    }

    /// Post-configuration lifecycle callback.
    ///
    /// Called regardless of whether config is loaded to indicate this is the
    /// time in app lifecycle when configuration would be loaded if
    /// possible.
    fn after_config(&mut self, config: Self::Cfg) -> Result<(), FrameworkError> {
        // Configure components
        self.state.components.after_config(&config)?;
        self.config = Some(config);
        Ok(())
    }

    /// Get logging configuration from command-line options
    fn logging_config(&self, command: &EntryPoint<CargoRpmCmd>) -> logging::Config {
        if command.verbose {
            logging::Config::verbose()
        } else {
            logging::Config::default()
        }
    }
}
