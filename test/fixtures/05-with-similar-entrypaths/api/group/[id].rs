use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let response = Response::builder()
        .status(StatusCode::IM_A_TEAPOT)
        .body(Body::Empty)
        .expect("Internal Server Error");

    Ok(response)
}
