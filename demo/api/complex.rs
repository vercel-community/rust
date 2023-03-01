use runtime_demo::choose_starter;
use serde_json::json;
use vercel_runtime::{
    lambda_http::{
        http::StatusCode, service_fn, tower::ServiceBuilder, Error as LambdaError, Response,
    },
    lambda_runtime, process_error, process_request, process_response, IntoResponse, ProxyError,
    ProxyRequest,
};

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::INFO)
        .with_max_level(tracing::Level::ERROR)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // Complex, using the ServiceBuilder to add middleware/layers
    let handler = ServiceBuilder::new()
        .map_request(process_request)
        .map_response(process_response)
        .map_err(process_error)
        .service(service_fn(handler));

    lambda_runtime::run(handler).await?;
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
