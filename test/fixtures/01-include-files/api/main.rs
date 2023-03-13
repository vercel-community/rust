use std::{env, fs::read_to_string, path::Path};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let current_dir = env::current_dir().unwrap();

    let file_path = Path::new(&current_dir).join("static/sample.txt");
    let text = read_to_string(file_path).unwrap();
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body(Body::Text(text))
        .expect("Internal Server Error");

    Ok(response)
}
