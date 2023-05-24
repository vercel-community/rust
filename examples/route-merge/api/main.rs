use serde_json::json;
use vercel_runtime::{bundled_api, run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

#[bundled_api(path = "examples/route-merge")]
pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    dbg!(req.uri());
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
