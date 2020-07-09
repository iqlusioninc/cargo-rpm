//! Derive rpm target architecture names from rust targets:
//! https://forge.rust-lang.org/release/platform-support.html
//! https://docs.fedoraproject.org/ro/Fedora_Draft_Documentation/0.1/html/RPM_Guide/ch01s03.html

#![allow(non_camel_case_types)]

use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};

/// Target architectures for which we have a defined rpm target architecture.
/// Only the architectures that vary in name between the rust and rpm target
/// need to be listed here explictly (others that follow the same naming across
/// both targets can be used as is).
pub enum TargetArch {
    /// mipsisa32r6
    mipsisa32r6,

    /// mipsisa32r6el
    mipsisa32r6el,

    /// mipsisa64r6
    mipsisa64r6,

    /// mipsisa64r6el
    mipsisa64r6el,

    /// powerpc
    powerpc,

    /// powerpc64
    powerpc64,

    /// powerpc64le
    powerpc64le,

    /// riscv64gc
    riscv64gc,

    /// x86
    x86,

    /// arm (hard-float)
    arm_hf,

    /// arm
    arm,

    /// for all other architectures
    Other(String),
}

impl TargetArch {
    /// Parse a specific rust target triple into the `TargetArch` enum
    pub fn parse(rust_target_triple: &str) -> Result<Self, Error> {
        let mut parts = rust_target_triple.split('-');
        let arch = parts.next().ok_or_else(|| {
            format_err!(
                ErrorKind::Parse,
                "no arch in the rust target {}!",
                rust_target_triple
            )
        })?;

        let abi = parts.last().unwrap_or("");

        // https://doc.rust-lang.org/std/env/consts/constant.ARCH.html
        // rustc --print target-list
        Ok(match (arch, abi) {
            ("mipsisa32r6", _) => TargetArch::mipsisa32r6,
            ("mipsisa32r6el", _) => TargetArch::mipsisa32r6el,
            ("mipsisa64r6", _) => TargetArch::mipsisa64r6,
            ("mipsisa64r6el", _) => TargetArch::mipsisa64r6el,
            ("powerpc", _) => TargetArch::powerpc,
            ("powerpc64", _) => TargetArch::powerpc64,
            ("powerpc64le", _) => TargetArch::powerpc64le,
            ("riscv64gc", _) => TargetArch::riscv64gc,
            ("x86", _) => TargetArch::x86,
            (arm, gnueabi) if arm.starts_with("arm") && gnueabi.ends_with("hf") => {
                TargetArch::arm_hf
            }
            (arm, _) if arm.starts_with("arm") => TargetArch::arm,
            (other_arch, _) => TargetArch::Other(other_arch.to_owned()),
        })
    }

    /// Return a rpm target architecture name
    pub fn as_rpm_target_architecture(&self) -> &str {
        // based on the similar function from cargo-deb:
        // https://github.com/mmstick/cargo-deb/blob/v1.23.1/src/manifest.rs#L909-L937
        // with adjustments for valid values that `rpmbuild --target` takes
        match self {
            // https://fedoraproject.org/wiki/Architectures
            // https://github.com/rpm-software-management/rpm/blob/rpm-4.14.3-release/rpmrc.in#L156
            TargetArch::mipsisa32r6 => "mipsr6",
            TargetArch::mipsisa32r6el => "mipsr6el",
            TargetArch::mipsisa64r6 => "mips64r6",
            TargetArch::mipsisa64r6el => "mips64r6el",
            TargetArch::powerpc => "ppc",
            TargetArch::powerpc64 => "ppc64",
            TargetArch::powerpc64le => "ppc64le",
            TargetArch::riscv64gc => "riscv64",
            TargetArch::x86 => "i386",
            TargetArch::arm_hf => "armv7hl",
            TargetArch::arm => "armv7l",
            TargetArch::Other(arch) => arch,
        }
    }
}
