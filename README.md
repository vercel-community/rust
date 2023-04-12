<p align="center">
  <a align="center" href="https://vercel.com">
    <img src="https://assets.vercel.com/image/upload/v1588805858/repositories/vercel/logo.png" height="96">
    <h3 align="center">Rust</h3>
  </a>
  <p align="center">Rust Runtime for Vercel Functions.</p>
</p>

<div align="center">

<a href="https://www.npmjs.com/package/vercel-rust">![npm version](https://img.shields.io/npm/v/vercel-rust.svg)</a>
<a href="https://www.npmjs.com/package/vercel-rust">![npm downloads](https://img.shields.io/npm/dm/vercel-rust.svg?label=npm%20downloads)</a>
<a href="https://crates.io/crates/vercel_runtime">![crates.io downloads](https://img.shields.io/crates/d/vercel_runtime?color=yellow&label=crates.io)</a>

Community-maintained package to support using [Rust](https://www.rust-lang.org/) inside [Vercel Functions](https://vercel.com/docs/serverless-functions/introduction) as a [Runtime](https://vercel.com/docs/runtimes).

</div>

## Legacy Runtime

The below documentation is for the `vercel_runtime` crate (in beta). If you are looking for the legacy runtime instructions using `vercel_lambda` see [tree/a9495a0](https://github.com/vercel-community/rust/tree/a9495a0f0d882a36ea165f1629fcc79c30bc3108).

## Introduction

This section aims to give you a basic understanding about Vercel runtimes.

### Builder module

The _npm_ module `vercel-rust` is implementing an interface which is primarily taking care of spawning a development server, caching between consecutive builds, and running the compilation. You can read more about the in-depths of implementing a builder [here](https://github.com/vercel/vercel/blob/main/DEVELOPING_A_RUNTIME.md).

Note that this dependency **does not** have to be installed manually, as it is pulled automatically by the Vercel CLI or the build container during deployments.

### Runtime crate

The crate `vercel_runtime` is what you will consume in your Rust functions. As the name suggests, the runtime crate takes care of everything that happens during run-time. In specific it takes care of creating a [Tower](https://docs.rs/tower/latest/tower/trait.Service.html) service, which expects a specific handler signature. The flow of an invocation can be visualized as the following:

```mermaid
graph TD
    A["Function Invocation"] --> |"process_request(event: InvocationEvent&lt;VercelEvent&gt;) → Request"| B[Request]
    B --> |"handler_fn(req: Request) → Future&lt;Output = Result&lt;Response&lt;Body&gt;, Error&gt;&gt;"| C["Runtime calls handler_fn"]
    C --> |"Ok(r) => process_response(r)"| D["Response"]
```

## Usage

First, you'll need a `vercel.json` file in your project and specify the Rust builder module for your Rust functions as a glob. As mentioned above, this dependency is pulled automatically and **does not** have to be installed.

In short this tells the builder that a Vercel Function should be created for every file that matches `api/**/*.rs`.

```json
{
  "functions": {
    "api/**/*.rs": {
      "runtime": "vercel-rust@4.0.0-beta.3"
    }
  }
}
```

Create your first function in `api`.

Example `api/handler.rs`

```rust
use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "message": "你好，世界"
            })
            .to_string()
            .into(),
        )?)
}
```

Now add a `Cargo.toml` file at the root of your repository.

```toml
# --snip--

[dependencies]
tokio = { version = "1", features = ["macros"] }
serde_json = { version = "1", features = ["raw_value"] }
# Documentation: https://docs.rs/vercel_runtime/latest/vercel_runtime
vercel_runtime = { version = "0.2.1" }

# You can specify a library for shared logic here (optional)
# [lib]
# path = "src-rs/lib.rs"

# Each handler has to be specified as [[bin]]
[[bin]]
name = "handler"
path = "api/handler.rs"

# Note that you need to provide unique names for each binary:
# [[bin]]
# name = "user-id"
# path = "api/user/[id].rs"
#
# [[bin]]
# name = "group-id"
# path = "api/group/[id].rs"

# --snip--
```

Finally make sure to ignore the build artifacts of Rust in your `.vercelignore`.

```shell
target/
```

### Local Development

With `vercel dev` you can develop your Rust-based lambdas on your own machine.

During local development with `vercel dev`, ensure `rust` and `cargo` are already installed and available in your `PATH`, since they will not be installed automatically. The recommended way to install is with [rustup](https://rustup.rs/).

### Dependencies

The builder module supports installing dependencies defined in the `Cargo.toml` file.

Furthermore, more system dependencies can be installed at build time with the presence of a shell `build.sh` file in the root directory of your project.

## Prebuilt Deployments

When creating a prebuilt deployment, the build output must be for `x86_64 linux`. To do this, create a Cargo build configuration at `.cargo/config.toml` with the following contents:

```toml
[build]
target = "x86_64-unknown-linux-musl"

# Uncomment below to support Rust cross-compilation from macOS to Linux
# Follow these installation instructions: https://github.com/chinedufn/cross-compile-rust-from-mac-to-linux
# [target.x86_64-unknown-linux-musl]
# linker = "x86_64-unknown-linux-gnu-gcc"
```

You then can build the file and trigger the deployment with the Vercel CLI.

```shell
vercel build && vercel deploy --prebuilt
```

## Contributing

Since this project contains both Rust and Node.js code, you need to install the relevant dependencies. If you're only working on the TypeScript side, you only need to install those dependencies (and vice-versa).

```shell
# install node dependencies
pnpm install

# install cargo dependencies
cargo fetch
```

## FAQ

<details>
  <summary>How to specify toolchain overrides</summary>

An example on how this can be achieved is using a `rust-toolchain` file adjacent to your `Cargo.toml`. Please refer to [Rust Documentation](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file) for more details.

</details>

<details>
  <summary>Can I use musl/static linking?</summary>
  
Unfortunately, the AWS Lambda Runtime for Rust relies (tangentially) on `proc_macro`, which won't compile on musl targets. Without `musl`, all linking must be dynamic. If you have a crate that relies on system libraries like `postgres` or `mysql`, you can include those library files with the `includeFiles` config option and set the proper environment variables, config, etc. that you need to get the library to compile.

For more information, please see [this issue](https://github.com/mike-engel/vercel-rust/issues/2).

</details>
