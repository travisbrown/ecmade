# ECMAde

[![build](https://github.com/travisbrown/ecmade/actions/workflows/ci.yml/badge.svg)](https://github.com/travisbrown/ecmade/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/travisbrown/ecmade/branch/main/graph/badge.svg)](https://codecov.io/gh/travisbrown/ecmade)
[![crates.io](https://img.shields.io/crates/v/ecmade.svg)](https://crates.io/crates/ecmade)
[![docs.rs](https://docs.rs/ecmade/badge.svg)](https://docs.rs/ecmade)

A [Serde][serde] deserializer for JavaScript, built on the [Speedy Web Compiler][swc]'s ECMAScript parsing library.

Functionality is currently limited to a small set of use cases, and only object literals, array literals, and a subset of scalar values are supported. The current error implementation is not useless, but could be organized better.

## License

This project is licensed under the [GNU General Public License, version 3
only](https://www.gnu.org/licenses/gpl-3.0.html). See [LICENSE](LICENSE) for the full text.

[serde]: https://serde.rs/
[swc]: https://swc.rs/
