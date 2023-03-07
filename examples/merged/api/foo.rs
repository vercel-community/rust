use runtime_demo::choose_starter;
use serde_json::json;
use vercel_runtime::{Body, Error, Request, Response, StatusCode};

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    dbg!(_req);
    let starter = choose_starter();
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "message": format!("I choose you, {}!", starter),
            })
            .to_string()
            .into(),
        )?;

    Ok(response)
}
