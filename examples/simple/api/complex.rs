use runtime_demo::choose_starter;
use serde::{Deserialize, Serialize};
use serde_json::json;
use vercel_runtime::{
    process_request, process_response, run_service, service_fn, Body, Error, Request, RequestExt,
    Response, ServiceBuilder, StatusCode,
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

#[derive(Debug, Serialize, Deserialize)]
struct Payload {
    trainer_name: String,
}

#[derive(Serialize)]
pub struct APIError {
    pub message: &'static str,
    pub code: &'static str,
}

impl From<APIError> for Body {
    fn from(val: APIError) -> Self {
        Body::Text(serde_json::to_string(&val).unwrap())
    }
}

pub fn bad_request(message: &'static str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("content-type", "application/json")
        .body(
            APIError {
                message,
                code: "bad_request",
            }
            .into(),
        )?)
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    tracing::info!("Choosing a starter Pokemon");
    let payload = req.payload::<Payload>();

    match payload {
        Err(..) => bad_request("Invalid payload"),
        Ok(None) => bad_request("Invalid payload"),
        Ok(Some(payload)) => {
            let starter = choose_starter();

            Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                json!({
                  "message": format!("{} says: I choose you, {}!", payload.trainer_name, starter),
                })
                .to_string()
                .into(),
            )?)
        }
    }
}
