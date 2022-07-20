# 🚨 UNMAINTAINED! 🚨

This crate is no longer maintained. For more information, please see the [maintenance status issue](https://github.com/iqlusioninc/cargo-rpm/issues/96).

We recommend either of the following as alternatives:

- [`cargo-generate-rpm`](https://github.com/cat-in-136/cargo-generate-rpm)
- [`rust2rpm`](https://pagure.io/fedora-rust/rust2rpm)

# cargo-rpm

[![Crate][crate-image]][crate-link]
[![Build Status][build-image]][build-link]
[![Safety Dance][safety-image]][safety-link]
![MSRV][rustc-image]
[![Apache 2.0 Licensed][license-image]][license-link]
[![Gitter Chat][gitter-image]][gitter-link]

[cargo subcommand] for building `.rpm` releases of Rust projects.

## Requirements

- Rust **1.41**+

## Installation

Install `cargo rpm` by running: `cargo install cargo-rpm`.

## Configuring a crate

To configure your crate for RPM releases, run `cargo rpm init`

This will create a `.rpm/YOURCRATENAME.spec` file which is passed to the
`rpmbuild` command. Though the generated spec should work out of the box,
it may need some customization if the resulting RPM has dependencies or
files other than target binaries.

You can also specify the `--output` argument to save the `.spec` file into
a different directory. However, you will then also need to add `config` entry
in the `[package.metadata.rpm]` section of the `Cargo.toml` file pointing to
that directory, or run `build` command with `--config` argument.

For more information on spec files, see:
<http://ftp.rpm.org/max-rpm/s1-rpm-build-creating-spec-file.html>

## Building RPMs

Once your crate has been configured, run `cargo rpm build` to build release
targets for your project and package them into an RPM.

If you encounter errors, you may need to see more information about why
`rpmbuild` failed. Run `cargo rpm build -v` to enable verbose mode.

Finished `.rpm` files will be placed in `target/release/rpmbuild/RPMs/<arch>`.

You can also specify the `--output` argument (or add the `output` entry in `Cargo.lock`)
to change the location of `.rpm` file. It can either be a file or a directory:

*  If the arg value ends in a `/` (or if it is already an existent directory), the value
   is treated as a directory path and rpm is created inside it with the default naming
   scheme (`<name>-<version>-<release>.<arch>.rpm`).
* For other arg values, the value is treated as a file path (the default naming scheme
  _won't_ be followed in this case).
* Both relative and absolute paths work as input (relative paths will be normalized to
  be absolute when passing over to `rpmbuild`).
* Parent directories in the path are auto-created, if not present (this is handled by
  `rpmbuild`).

## License

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/cargo-rpm.svg
[crate-link]: https://crates.io/crates/cargo-rpm
[build-image]: https://github.com/iqlusioninc/cargo-rpm/workflows/rust/badge.svg?branch=develop&event=push
[build-link]: https://github.com/iqlusioninc/cargo-rpm/actions
[rustc-image]: https://img.shields.io/badge/rustc-1.41+-blue.svg
[safety-image]: https://img.shields.io/badge/unsafe-forbidden-success.svg
[safety-link]: https://github.com/rust-secure-code/safety-dance/
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/rustrpm/cargo-rpm/blob/master/LICENSE
[gitter-image]: https://badges.gitter.im/badge.svg
[gitter-link]: https://gitter.im/RustRPM/communit

[//]: # (general links)

[cargo subcommand]: https://github.com/rust-lang/cargo/wiki/Third-party-cargo-subcommands
