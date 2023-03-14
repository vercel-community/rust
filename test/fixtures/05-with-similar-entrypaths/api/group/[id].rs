use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .expect("Internal Server Error");

    Ok(response)
}
