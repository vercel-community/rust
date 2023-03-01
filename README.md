<p align="center">
  <a href="https://vercel.com">
    <img src="https://assets.vercel.com/image/upload/v1588805858/repositories/vercel/logo.png" height="96">
    <h3 align="center">Rust</h3>
  </a>
  <p align="center">Rust runtime for Vercel Functions.</p>
</p>

Community-maintained package to support using [Rust](https://www.rust-lang.org/) inside [Vercel Functions](https://vercel.com/docs/serverless-functions/introduction) as a [Runtime](https://vercel.com/docs/runtimes).

## Usage

First, you'll need a `vercel.json` file in your project:

```json
{
  "functions": {
    "api/**/*.rs": {
      "runtime": "vercel-rust@4.0.0-canary.0"
    }
  }
}
```

A Vercel Function will be created for every file that matches `api/**/*.rs`.

Example:

```rust
use serde_json::json;
use vercel_runtime::{
    lambda_http::{http::StatusCode, Error as LambdaError, Response},
    run, IntoResponse, ProxyError, ProxyRequest,
};

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    run(handler).await?;
    Ok(())
}

pub async fn handler(_req: ProxyRequest) -> Result<impl IntoResponse, ProxyError> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "message": "你好，世界"
            })
            .to_string(),
        )?;

    Ok(response)
}
```

Finally we need a `Cargo.toml` file at the root of your repository.

```toml
# You can specify a library for shared logic here (optional)
# [lib]
# path = "src-rs/lib.rs"

# Each handler has to be specified as [[bin]]
[[bin]]
name = "handler"
path = "api/handler.rs"
```

### Dependencies

This Builder supports installing dependencies defined in the `Cargo.toml` file.

Furthermore, more system dependencies can be installed at build time with the presence of a shell `build.sh` file in the same directory as the entrypoint file.

## Local Development

With `vercel dev` and `vercel-rust`, you can develop your Rust-based lambdas on your own machine.

During local development with `vercel dev`, ensure `rust` and `cargo` are already installed and available in your `PATH`, since they will not be installed automatically. The recommended way to install is with [rustup](https://rustup.rs/).

## Contributing

Since this project contains both Rust and Node.js code, you need to install the relevant dependencies. If you're only working on the JavaScript side, you only need to install those dependencies (and vice-versa).

```
# install node dependencies
npm install

# install cargo dependencies
cargo fetch
```

## FAQ

<details>
  <summary>Are cargo workspaces supported?</summary>
  
Not quite. Cargo's workspaces feature is a great tool when working on multiple binaries and libraries in a single project. If a cargo workspace is found in the entrypoint, however, `vercel-rust` will fail to build.

To get around this limitation, create build entries in your `vercel.json` file for each `Cargo.toml` that represents a Function within your workspace. In your `.vercelignore`, you'll want to add any binary or library project folders that aren't needed for your lambdas to speed up the build process like your `Cargo.toml` workspace.

It's also recommended to have a `Cargo.lock` alongside your lambda `Cargo.toml` files to speed up the build process. You can do this by running cargo check or a similar command within each project folder that contains a lambda.

If you have a compelling case for workspaces to be supported by `vercel-rust` which are too cumbersome with this workaround, please submit an issue! We're always looking for feedback.

</details>

<details>
  <summary>Can I use musl/static linking?</summary>
  
Unfortunately, the AWS Lambda Runtime for Rust relies (tangentially) on `proc_macro`, which won't compile on musl targets. Without `musl`, all linking must be dynamic. If you have a crate that relies on system libraries like `postgres` or `mysql`, you can include those library files with the `includeFiles` config option and set the proper environment variables, config, etc. that you need to get the library to compile.

For more information, please see [this issue](https://github.com/mike-engel/vercel-rust/issues/2).

</details>
