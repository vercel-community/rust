# vercel-rust changelog

## 4.0.9

- Fix select binary correctly while vercel dev [#184](https://github.com/vercel-community/rust/pull/184)

## 4.0.8

- Use debug build for `vercel dev` [#168](https://github.com/vercel-community/rust/pull/168)

## 4.0.7

- Use correct provided runtime [#155](https://github.com/vercel-community/rust/pull/155)

## 4.0.6

- Do not include main route when bundling functions [#117](https://github.com/vercel-community/rust/pull/117)

## 4.0.5

- Support function options [#116](https://github.com/vercel-community/rust/pull/116)

## 4.0.4

- Unpublished - Accidental publish

## 4.0.3

- Set cargo metadata to run in `workPath` [#111](https://github.com/vercel-community/rust/pull/111)

## 4.0.2

- Fix `HOME` env var not exist on Windows [#107](https://github.com/vercel-community/rust/pull/107)

## 4.0.1

- Address routing issues with invalid dest [#102](https://github.com/vercel-community/rust/pull/102)

## 4.0.0

- First stable release for new Rust runtime crates
- Includes support for experimental route merging.

## 3.1.0

- Support pre-configured binaries [#25](https://github.com/mike-engel/vercel-rust/pull/25)

## 3.0.0

- Renamed to `vercel-rust` [#34](https://github.com/mike-engel/vercel-rust/pull/34)

## 2.0.3

- Fix issue with last PR where a variable wasn't defined

## 2.0.2

- Restore `cargo.toml` when the build fails during dev #29

## 2.0.1

Update dependencies

## 2.0.0

Big thanks to [ekadas](https://github.com/ekadas) for [fixing a ton of issues](https://github.com/mike-engel/vercel-rust/pull/19)!

## Unreleased

## 4.0.0.beta.4

- Update documentation #94
- Add cargo build config and prebuilt support #91
- Prevent leaking source code in examples #90
- Add tests and docs for configurable toolchain overrides #89

## 4.0.0.beta.3

- Execute cargo build inside `workPath` #85

## 4.0.0.beta.2

- Fix executable resolving when using dev server on Windows

## 4.0.0.beta.1

- Fix to support build targets with different name than its path filename

## 4.0.0.beta.0

- New builder and runtime crate (published as `vercel_runtime`)
- Reworked tests
- Added examples

### Breaking changes

- OpenSSL is no longer installed by default
- Platform version 1 is no longer supported
- `Cargo.toml` is no longer a valid entrypoint

### Bug fixes

- `now dev` is now 100% functional :tada:

## 1.1.2

- Update dependencies, expose types [#160](https://github.com/vercel-community/rust/pull/160)

## 1.1.1

- Fix bundled_api glob [#145](https://github.com/vercel-community/rust/pull/145)

## 1.1.0

- Hide requestExt [#120](https://github.com/vercel-community/rust/pull/120)

## 1.0.1

- Support for version 3 of Runtimes [#14](https://github.com/mike-engel/vercel-rust/pull/14)

## 0.2.6

- Rust will be installed through a now lifecycle hook in `package.json` rather than by the builder [#5](https://github.com/mike-engel/vercel-rust/pull/5)

## 0.2.5

This is the initial release as a community-maintained repository. It includes all the existing data from the old official builder as well as the docs from the Vercel Builder page.

For previous version, please see the [old builder](https://github.com/vercel/now-builders) repo.
