# ECMAde

[![Rust build status](https://img.shields.io/github/actions/workflow/status/travisbrown/ecmade/ci.yaml?branch=main)](https://github.com/travisbrown/ecmade/actions)
[![Coverage status](https://img.shields.io/codecov/c/github/travisbrown/ecmade/main.svg)](https://codecov.io/github/travisbrown/ecmade)

A [Serde][serde] deserializer for JavaScript, built on the [Speedy Web Compiler][swc]'s ECMAScript parsing library.

Functionality is currently limited to a small set of use cases, and only object literals, array literals, and a subset of scalar values are supported. The current error implementation is not useless, but could be organized better.

[serde]: https://serde.rs/
[swc]: https://swc.rs/
