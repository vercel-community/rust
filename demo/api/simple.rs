use runtime_demo::choose_starter;
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
