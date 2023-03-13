use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use integration_test::return_foo;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body(Body::Text(return_foo()))
        .expect("Internal Server Error");

    Ok(response)
}
