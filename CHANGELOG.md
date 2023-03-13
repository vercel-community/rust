# vercel-rust changelog

## Unreleased

## 4.0.0.beta.0

- New builder and runtime crate (published as `vercel_runtime`)
- Reworked tests
- Added examples

## 3.1.0

- Support pre-configured binaries [#25](https://github.com/mike-engel/vercel-rust/pull/25)

## 3.0.0

- Renamed to `vercel-rust` [#34](https://github.com/mike-engel/vercel-rust/pull/34)

## 2.0.3

### Bug fixes

- Fix issue with last PR where a variable wasn't defined

## 2.0.2

### Bug fixes

- Restore `cargo.toml` when the build fails during dev #29

## 2.0.1

Update dependencies

## 2.0.0

Big thanks to [ekadas](https://github.com/ekadas) for [fixing a ton of issues](https://github.com/mike-engel/vercel-rust/pull/19)!

### Breaking changes

- OpenSSL is no longer installed by default
- Platform version 1 is no longer supported
- `Cargo.toml` is no longer a valid entrypoint

### Bug fixes

- `now dev` is now 100% functional :tada:

## 1.0.1

### New features

- Support for version 3 of Runtimes [#14](https://github.com/mike-engel/vercel-rust/pull/14)

## 0.2.6

### New features

- Rust will be installed through a now lifecycle hook in `package.json` rather than by the builder [#5](https://github.com/mike-engel/vercel-rust/pull/5)

## 0.2.5

This is the initial release as a community-maintained repository. It includes all the existing data from the old official builder as well as the docs from the Vercel Builder page.

For previous version, please see the [old builder](https://github.com/vercel/now-builders) repo.
