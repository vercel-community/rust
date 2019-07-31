# now-rust

> Community based builder for using rust on the now/zeit platform

This is a now builder which allows you to run your rust code as lambdas on the now platform!

This was originally provided officially by [ZEIT](https://zeit.co)'s [now-builders](https://github.com/zeit/now-builders) monorepo, but has since been moved to a community-maintained project.

## Usage

If you're unfamiliar with now builders, please read the [builder docs](https://zeit.co/docs/v2/advanced/builders/overview/) first. To use this builder, you can use it as you would use any other builder.

```json
{
	"version": 2,
	"builds": [{ "src": "Cargo.toml", "use": "now-rust" }]
}
```

That's the simplest way to use this builder! Below you'll find more complex and advanced patterns.

### Entry point

The entry point file can either be a `.rs` source file or a `Cargo.toml` file.

#### `.rs` entrypoint

When you use one or multiple `.rs` files as an entry point for this Builder, Now will setup the serverless environment for you.

The requirements for this entry point is to expose a `handler` function and not to have a `main` function.

If a `Cargo.toml` exists in the project relating to the entry point, the dependencies will be installed for the Rust project.

#### `Cargo.toml` entrypoint

When using a `Cargo.toml` file as an entry point for this Builder, Now will use `cargo read-manifest` to build each binary within the project. As a result, **[cargo workspaces`](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) are not supported as an entry point for Now**—you should read the [cargo workspace workaround](#are-cargo-workspaces-supported) for further information.

This entry point method is an advanced method of using this Builder and requires Rust files to assemble their own runtimes.

Defining a `Cargo.toml` file as an entry point requires a Rust file at `src/main.rs` or files defined as a [`[[bin]]`](https://doc.rust-lang.org/cargo/reference/manifest.html#configuring-a-target) target.

An example `src/main.rs` Rust file within a project including a `Cargo.toml` file acting as the entry point:

```rs
use http::{StatusCode, header};
use now_lambda::{error::NowError, lambda, IntoResponse, Request, Response};
use std::error::Error;

fn handler(request: Request) -> Result<impl IntoResponse, NowError> {
	let uri = request.uri();
	let response = Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "text/html")
		.body(format!(
				"You made a request to the following URL: {}",
				uri
		))
		.expect("failed to render response");

	Ok(response)
}

// Start the runtime with the handler
fn main() -> Result<(), Box<dyn Error>> {
	Ok(lambda!(handler))
}
```

This requires one dependency, with the example above using another dependency, `http`.

The required dependency is `now_lambda` which provides all of the resources needed to provide the serverless runtime.

The `Cargo.toml` entry point for the example above is the following:

```toml
[package]
name = "rust-project"
version = "0.1.0"
edition = "2018"

[dependencies]
http = "0.1"
now_lambda = "0.1"
```

### Rust version

This builder uses [rustup](https://rustup.rs) to install `rust` and `cargo`. By default, the latest stable version of rust will be installed. To see what the current stable version of rust is, please see the [official website](https://www.rust-lang.org).

If you need to use a different version of rust other than the latest stable version, you can specify a version of rust in your build's configuration. Accepted values are the same as [rustup's channel definition](https://github.com/rust-lang/rustup.rs/#toolchain-specification), which is `stable | latest | nightly | <version>`.

```json
{
	"version": 2,
	"builds": [
		{ "src": "Cargo.toml", "use": "now-rust", "config": { "rust": "1.31" } }
	]
}
```

### Dependencies

This Builder supports installing dependencies defined in the `Cargo.toml` file.

Furthermore, more system dependencies can be installed at build time with the presence of a shell `build.sh` file in the same directory as the entry point file.

By default, `openssl` is installed by the Builder due to its common usage with Rust projects.

## FAQ

### Are cargo workspaces supported?

Not quite. Cargo's workspaces feature is a great tool when working on multiple binaries and libraries in a single project. If a cargo workspace is found in the entrypoint, however, now-rust will fail to build.

To get around this limitation, create build entries in your now.json file for each Cargo.toml that represents a lambda function within your workspace. In your .nowignore, you'll want to add any binary or library project folders that aren't needed for your lambdas to speed up the build process like your Cargo.toml workspace.

It's also recommended to have a Cargo.lock alongside your lambda Cargo.toml files to speed up the build process. You can do this by running cargo check or a similar command within each project folder that contains a lambda.

If you have a compelling case for workspaces to be supported by now-rust which are too cumbersome with this workaround, please submit an issue! We're always looking for feedback.

### Development

The `now dev` command allows you to develop lambdas locally on your machine. With `now dev` and `now-rust` you can develop your rust-based lamdas on your own machine.

During local development with `now dev`, the assumption is that `rust` and `cargo` are already installed and available in your `PATH` since they will not be installed automatically. The recommended way to install `rust` and `cargo` on your machine is with [rustup](https://rustup.rs).

## Contibuting

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

Issues and pull requests are welcome!

### Setup

Since this project contains both rust and node code, you need to install the relevant dependencies. If you're only working on the javascript side, you only need to install those dependencies. The oppoosite is true for the rust side.

```sh
# install node dependencies
npm install

# install cargo dependencies
cargo fetch
```

At this point, you're all set up and can start making edits!

**Note: During the migration period, tests will be broken until we get CI set up!**

## License

[MIT](LICENSE.md)

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore -->
<table>
	<tr>
		<td align="center"><a href="https://www.mike-engel.com"><img src="https://avatars0.githubusercontent.com/u/464447?v=4" width="100px;" alt="Mike Engel"/><br /><sub><b>Mike Engel</b></sub></a><br /><a href="#question-mike-engel" title="Answering Questions">💬</a> <a href="https://github.com/Mike Engel <mike@mike-engel.com>/now-rust/commits?author=mike-engel" title="Code">💻</a> <a href="https://github.com/Mike Engel <mike@mike-engel.com>/now-rust/commits?author=mike-engel" title="Documentation">📖</a> <a href="#example-mike-engel" title="Examples">💡</a> <a href="https://github.com/Mike Engel <mike@mike-engel.com>/now-rust/commits?author=mike-engel" title="Tests">⚠️</a> <a href="#review-mike-engel" title="Reviewed Pull Requests">👀</a> <a href="#maintenance-mike-engel" title="Maintenance">🚧</a> <a href="#design-mike-engel" title="Design">🎨</a> <a href="#infra-mike-engel" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#ideas-mike-engel" title="Ideas, Planning, & Feedback">🤔</a> <a href="#content-mike-engel" title="Content">🖋</a></td>
		<td align="center"><a href="https://twitter.com/_anmonteiro"><img src="https://avatars2.githubusercontent.com/u/661909?v=4" width="100px;" alt="Antonio Nuno Monteiro"/><br /><sub><b>Antonio Nuno Monteiro</b></sub></a><br /><a href="#question-anmonteiro" title="Answering Questions">💬</a> <a href="https://github.com/Mike Engel <mike@mike-engel.com>/now-rust/commits?author=anmonteiro" title="Code">💻</a> <a href="https://github.com/Mike Engel <mike@mike-engel.com>/now-rust/commits?author=anmonteiro" title="Documentation">📖</a> <a href="#example-anmonteiro" title="Examples">💡</a> <a href="https://github.com/Mike Engel <mike@mike-engel.com>/now-rust/commits?author=anmonteiro" title="Tests">⚠️</a> <a href="#review-anmonteiro" title="Reviewed Pull Requests">👀</a> <a href="#maintenance-anmonteiro" title="Maintenance">🚧</a> <a href="#design-anmonteiro" title="Design">🎨</a> <a href="#infra-anmonteiro" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#ideas-anmonteiro" title="Ideas, Planning, & Feedback">🤔</a> <a href="#content-anmonteiro" title="Content">🖋</a></td>
	</tr>
</table>

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!
