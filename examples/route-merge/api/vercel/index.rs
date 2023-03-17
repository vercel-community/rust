use serde_json::json;
use vercel_runtime::{include_api, run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

#[include_api]
pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "code": "not_found",
              "message": "not_found"
            })
            .to_string()
            .into(),
        )?)
}
