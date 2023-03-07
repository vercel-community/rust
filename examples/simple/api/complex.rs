use runtime_demo::choose_starter;
use serde_json::json;
use vercel_runtime::{
    process_request, process_response, run_service, service_fn, Body, Error, Request, Response,
    ServiceBuilder, StatusCode,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        // disable printing the name of the module in every log line.
        .with_target(false)
        .init();

    // This allows to extend the tower service with more layers
    let handler = ServiceBuilder::new()
        .map_request(process_request)
        .map_response(process_response)
        .service(service_fn(handler));

    run_service(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    tracing::info!("Choosing a starter Pokemon");
    let starter = choose_starter();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "message": format!("I choose you, {}!", starter),
            })
            .into(),
        )?;

    Ok(response)
}
