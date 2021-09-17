pub use http::{self, Response};
use lambda_runtime::{self as lambda, handler_fn, Context};
use log::{self, debug, error};
use serde_json::Error;
use std::future::Future;

mod body;
pub mod error;
pub mod request;
mod response;
mod strmap;

pub use crate::{body::Body, response::IntoResponse, strmap::StrMap};
use crate::{
    error::VercelError,
    request::{VercelEvent, VercelRequest},
    response::VercelResponse,
};

/// Type alias for `http::Request`s with a fixed `Vercel_lambda::Body` body
pub type Request = http::Request<Body>;

/// Creates a new `lambda_runtime::Runtime` and begins polling for Vercel Lambda events
///
/// # Arguments
///
/// * `f` function pointer type `fn(http::Request<B>) -> Result<R, E>`
///
/// # Panics
/// The function panics if the Lambda environment variables are not set.
#[cfg(feature = "runtime")]
#[inline]
pub fn start<R, B, E>(
    f: fn(http::Request<B>) -> Result<R, E>,
    runtime: Option<tokio::runtime::Runtime>,
) where
    B: From<Body>,
    E: Into<VercelError>,
    R: IntoResponse,
{
    let runtime = get_runtime(runtime);
    runtime.block_on(start_handler_async(f)).unwrap();
}

#[inline]
pub async fn start_handler_async<R, B, E>(
    f: fn(http::Request<B>) -> Result<R, E>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    B: From<Body>,
    E: Into<VercelError>,
    R: IntoResponse,
{
    let func = |e: VercelEvent, ctx: Context| process_vercel_request(f, e, ctx);
    let func = handler_fn(func);
    lambda::run(func).await
}

async fn process_vercel_request<R, B, E>(
    f: fn(http::Request<B>) -> Result<R, E>,
    e: VercelEvent,
    _ctx: Context,
) -> Result<VercelResponse, VercelError>
where
    B: From<Body>,
    E: Into<VercelError>,
    R: IntoResponse,
{
    let parse_result: Result<VercelRequest, Error> = serde_json::from_str(&e.body);
    match parse_result {
        Ok(req) => {
            debug!("Deserialized Vercel proxy request successfully");
            let request: Request = req.into();
            let request = request.map(|b| b.into());
            f(request)
                .map(|resp| VercelResponse::from(resp.into_response()))
                .map_err(|e| e.into())
        }
        Err(e) => {
            error!("Could not deserialize event body to VercelRequest {}", e);
            panic!("Could not deserialize event body to VercelRequest {}", e);
        }
    }
}

/// Creates a new `lambda_runtime::Runtime` and begins polling for Vercel Lambda events
///
/// # Arguments
///
/// * `f` function pointer type `fn(http::Request<B>) -> Future<Output = Result<R, E>>`
///
/// # Panics
/// The function panics if the Lambda environment variables are not set.
#[cfg(feature = "runtime")]
#[inline]
pub fn start_async_handler<R, B, E, F>(
    f: fn(http::Request<B>) -> F,
    runtime: Option<tokio::runtime::Runtime>,
) where
    F: Future<Output = Result<R, E>>,
    B: From<Body>,
    E: Into<VercelError>,
    R: IntoResponse,
{
    let runtime = get_runtime(runtime);
    runtime.block_on(start_async_handler_async(f)).unwrap();
}

#[inline]
pub async fn start_async_handler_async<R, B, E, F>(
    f: fn(http::Request<B>) -> F,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Future<Output = Result<R, E>>,
    B: From<Body>,
    E: Into<VercelError>,
    R: IntoResponse,
{
    let func = |e: VercelEvent, ctx: Context| process_vercel_request_with_async_handler(f, e, ctx);
    let func = handler_fn(func);
    lambda::run(func).await
}

async fn process_vercel_request_with_async_handler<R, B, E, F>(
    f: fn(http::Request<B>) -> F,
    e: VercelEvent,
    _ctx: Context,
) -> Result<VercelResponse, VercelError>
where
    F: Future<Output = Result<R, E>>,
    B: From<Body>,
    E: Into<VercelError>,
    R: IntoResponse,
{
    let parse_result: Result<VercelRequest, Error> = serde_json::from_str(&e.body);
    match parse_result {
        Ok(req) => {
            debug!("Deserialized Vercel proxy request successfully");
            let request: Request = req.into();
            let request: http::Request<B> = request.map(|b| b.into());
            let future = f(request);
            let r = future.await;
            r.map(|resp| VercelResponse::from(resp.into_response()))
                .map_err(|e| e.into())
        }
        Err(e) => {
            error!("Could not deserialize event body to VercelRequest {}", e);
            panic!("Could not deserialize event body to VercelRequest {}", e);
        }
    }
}

#[cfg(feature = "runtime")]
#[inline]
fn get_runtime(runtime: Option<tokio::runtime::Runtime>) -> tokio::runtime::Runtime {
    match runtime {
        Some(rt) => rt,
        _ => tokio::runtime::Builder::new_multi_thread()
            .build()
            .unwrap_or_else(|_| {
                tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap()
            }),
    }
}

/// A macro for starting new handler's poll for Vercel Lambda events
#[cfg(feature = "runtime")]
#[macro_export]
macro_rules! lambda {
    ($handler:expr) => {
        $crate::start($handler, None)
    };
    ($handler:expr, $runtime:expr) => {
        $crate::start($handler, Some($runtime))
    };
    ($handler:ident) => {
        $crate::start($handler, None)
    };
    ($handler:ident, $runtime:expr) => {
        $crate::start($handler, Some($runtime))
    };
}

/// A macro for starting new async handler's poll for Vercel Lambda events
#[cfg(feature = "runtime")]
#[macro_export]
macro_rules! lambda_async {
    ($handler:expr) => {
        $crate::start_async_handler($handler, None)
    };
    ($handler:expr, $runtime:expr) => {
        $crate::start_async_handler($handler, Some($runtime))
    };
    ($handler:ident) => {
        $crate::start_async_handler($handler, None)
    };
    ($handler:ident, $runtime:expr) => {
        $crate::start_async_handler($handler, Some($runtime))
    };
}

#[cfg(test)]
mod test {
    use crate::{error::VercelError, *};
    #[test]
    fn handler_compile_test() {
        fn _my_handler(_request: Request) -> Result<impl IntoResponse, VercelError> {
            let response = Response::builder()
                .status(200)
                .header("Content-Type", "text/plain")
                .body("Hello world")
                .expect("Internal Server Error");
            Ok(response)
        }

        #[cfg(feature = "runtime")]
        fn _main() -> Result<(), Box<dyn std::error::Error>> {
            Ok(lambda!(_my_handler))
        }

        #[tokio::main]
        async fn _async_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            start_handler_async(_my_handler).await
        }
    }

    #[test]
    fn async_handler_compile_test() {
        async fn _my_async_handler(_request: Request) -> Result<impl IntoResponse, VercelError> {
            let response = Response::builder()
                .status(200)
                .header("Content-Type", "text/plain")
                .body("Hello world")
                .expect("Internal Server Error");
            Ok(response)
        }

        #[cfg(feature = "runtime")]
        fn _main() -> Result<(), Box<dyn std::error::Error>> {
            Ok(lambda_async!(_my_async_handler))
        }

        #[tokio::main]
        async fn _async_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            start_async_handler_async(_my_async_handler).await
        }
    }
}
