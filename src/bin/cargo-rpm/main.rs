//! Main entry point for `cargo rpm`

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use cargo_rpm::application::APPLICATION;

/// Boot `cargo-rpm`
fn main() {
    abscissa_core::boot(&APPLICATION);
}
