pub mod http;
pub mod request;
pub mod response;

use request::Event;
use request::VercelRequest;
use response::EventResponse;
use std::future::Future;
use tracing::{debug, error};

pub use vercel_runtime_macro::bundled_api;
pub use vercel_runtime_router::{Route, Router};

pub use lambda_http::{
    http::StatusCode, service_fn, tower::ServiceBuilder, Body, Error, Request, RequestPayloadExt,
    Response,
};
pub use lambda_runtime::run as run_service;

pub async fn run<T: FnMut(Request) -> F, F: Future<Output = Result<Response<Body>, Error>>>(
    f: T,
) -> Result<(), Error> {
    let handler = ServiceBuilder::new()
        .map_request(process_request)
        .map_response(process_response)
        .service(service_fn(f));

    lambda_runtime::run(handler).await
}

pub fn process_request(event: Event) -> Request {
    let (event, _context) = event.into_parts();
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

pub fn process_response(response: Response<Body>) -> EventResponse {
    EventResponse::from(response)
}
