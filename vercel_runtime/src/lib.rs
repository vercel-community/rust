pub mod body;
pub mod request;
pub mod response;
use body::Body;
pub use lambda_http;
use lambda_http::{service_fn, tower::ServiceBuilder};
pub use lambda_runtime;
use lambda_runtime::{Error as RuntimeError, LambdaEvent};
use request::{VercelEvent, VercelRequest};
pub use response::IntoResponse;
use response::VercelResponse;
use std::future::Future;
use tracing::{debug, error};

pub type ProxyRequest = lambda_http::http::Request<Body>;
pub type ProxyError = lambda_http::http::Error;

pub async fn run<
    T: FnMut(ProxyRequest) -> F,
    F: Future<Output = Result<impl IntoResponse, ProxyError>>,
>(
    f: T,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let handler = ServiceBuilder::new()
        .map_request(process_request)
        .map_response(process_response)
        .map_err(process_error)
        .service(service_fn(f));

    lambda_runtime::run(handler).await
}

pub fn process_request(lambda_event: LambdaEvent<VercelEvent>) -> lambda_http::http::Request<Body> {
    let (event, _context) = lambda_event.into_parts();
    let parse_result = serde_json::from_str::<VercelRequest>(&event.body);

    match parse_result {
        Ok(request) => {
            debug!("Deserialized Vercel proxy request successfully");
            debug!("Request: {:?}", request);
            let http_req: lambda_http::http::Request<Body> = request.into();
            http_req.map(|b| b)
        }
        Err(e) => {
            error!("Could not deserialize event body to VercelRequest {:?}", e);
            panic!("Could not deserialize event body to VercelRequest {}", e);
        }
    }
}

pub fn process_response(response: impl IntoResponse) -> VercelResponse {
    VercelResponse::from(response.into_response())
}

pub fn process_error(error: lambda_http::http::Error) -> RuntimeError {
    error.into()
}
