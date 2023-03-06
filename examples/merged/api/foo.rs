use runtime_demo::choose_starter;
use serde_json::json;
use vercel_runtime::{
    lambda_http::{http::StatusCode, Response},
    Error, IntoResponse, Request,
};

pub async fn handler(_req: Request) -> Result<impl IntoResponse, Error> {
    dbg!(_req);
    let starter = choose_starter();
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "message": format!("I choose you, {}!", starter),
            })
            .to_string(),
        )?;

    Ok(response)
}
