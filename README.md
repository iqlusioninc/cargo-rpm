# cargo rpm

[![Crate][crate-image]][crate-link]
[![Build Status][build-image]][build-link]
[![Apache 2.0 Licensed][license-image]][license-link]

[crate-image]: https://img.shields.io/crates/v/cargo-rpm.svg
[crate-link]: https://crates.io/crates/cargo-rpm
[build-image]: https://circleci.com/gh/RustRPM/cargo-rpm.svg?style=shield
[build-link]: https://circleci.com/gh/RustRPM/cargo-rpm
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/rustrpm/cargo-rpm/blob/master/LICENSE

A [cargo subcommand] for building `.rpm` releases of Rust projects.

[cargo subcommand]: https://github.com/rust-lang/cargo/wiki/Third-party-cargo-subcommands

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
