//! Acceptance test: runs the application as a subprocess and asserts its
//! output for given argument combinations matches what is expected.
//!
//! For more information, see:
//! <https://docs.rs/abscissa_core/latest/abscissa_core/testing/index.html>

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use abscissa_core::testing::prelude::*;
use once_cell::sync::Lazy;

pub static RUNNER: Lazy<CmdRunner> = Lazy::new(CmdRunner::default);

/// Test the `cargo rpm version` subcommand
#[test]
fn version_no_args() {
    let mut runner = RUNNER.clone();
    let mut cmd = runner.args(&["rpm", "version"]).capture_stdout().run();
    cmd.stdout().expect_regex(r"\Acargo-rpm [\d\.\-]+\z");
}
