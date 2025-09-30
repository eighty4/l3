# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- `build_fn` loads `tsconfig.json` and compiles .ts to .js
- `FnBuildManifest` maps input sources to outputs with a new `paths` map
  found in the `FnBuildOutput`
- `FnOutputConfig` has a new param `dirname` used to build out dir instead of
  dynamically building the dirname from the function's routing to keep `l3_fn_build`
  API agnostic of its integration or upstream tooling

## 0.0.4 - 2024-01-02

### Added

- Windows fs path compatibility

## 0.0.3 - 2024-12-20

## 0.0.2 - 2024-12-20

### Added

- FnEntrypoint::to_fn_identifier creates a unique identifier for a function

## 0.0.1

### Added

- Parse and build APIs for creating deployable Lambda functions

[Unreleased]: https://github.com/eighty4/l3/compare/l3_fn_build-v0.0.4...HEAD
[0.0.4]: https://github.com/eighty4/l3/compare/l3_fn_build-v0.0.3...l3_fn_build-v0.0.4
[0.0.3]: https://github.com/eighty4/l3/compare/l3_fn_build-v0.0.2...l3_fn_build-v0.0.3
[0.0.2]: https://github.com/eighty4/l3/compare/v0.0.1...l3_fn_build-v0.0.2
[0.0.1]: https://github.com/eighty4/l3/releases/tag/v0.0.1
