use serde::Serialize;
use std::collections::HashMap;
use url::Url;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
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
    let parsed_url = Url::parse(&req.uri().to_string()).unwrap();
    let hash_query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    let id_key = hash_query.get("id");

    match id_key {
        None => {
            return bad_request("Invalid query string");
        }
        Some(id) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::Text(id.to_owned()))?),
    }
}
