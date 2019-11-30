# cargo-rpm

[![Crate][crate-image]][crate-link]
[![Build Status][build-image]][build-link]
[![Safety Dance][safety-image]][safety-link]
![MSRV][rustc-image]
[![Apache 2.0 Licensed][license-image]][license-link]
[![Gitter Chat][gitter-image]][gitter-link]

[cargo subcommand] for building `.rpm` releases of Rust projects.

## Requirements

- Rust **1.36**+

## Installation

Install `cargo rpm` by running: `cargo install cargo-rpm`.

## Configuring a crate

To configure your crate for RPM releases, run `cargo rpm init`

This will create a `.rpm/YOURCRATENAME.spec` file which is passed to the
`rpmbuild` command. Though the generated spec should work out of the box,
it may need some customization if the resulting RPM has dependencies or
files other than target binaries.

For more information on spec files, see:
<http://ftp.rpm.org/max-rpm/s1-rpm-build-creating-spec-file.html>

## Building RPMs

Once your crate has been configured, run `cargo rpm build` to build release
targets for your project and package them into an RPM.

If you encounter errors, you may need to see more information about why
`rpmbuild` failed. Run `cargo rpm build -v` to enable verbose mode.

Finished `.rpm` files will be placed in `target/release/rpmbuild/RPMs/<arch>`

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
[build-image]: https://travis-ci.org/RustRPM/cargo-rpm.svg?branch=master
[build-link]: https://travis-ci.org/RustRPM/cargo-rpm
[rustc-image]: https://img.shields.io/badge/rustc-1.36+-blue.svg
[safety-image]: https://img.shields.io/badge/unsafe-forbidden-success.svg
[safety-link]: https://github.com/rust-secure-code/safety-dance/
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/rustrpm/cargo-rpm/blob/master/LICENSE
[gitter-image]: https://badges.gitter.im/badge.svg
[gitter-link]: https://gitter.im/RustRPM/communit

[//]: # (general links)

[cargo subcommand]: https://github.com/rust-lang/cargo/wiki/Third-party-cargo-subcommands
