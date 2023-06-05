use std::collections::HashMap;

use url::Url;
use vercel_runtime::{Body, Error, Request, Response, StatusCode};

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let parsed_url = Url::parse(&req.uri().to_string()).unwrap();
    let hash_query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    let query_parameter = hash_query.get("path");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::Text(match query_parameter {
            Some(query_parameter) => format!(
                "Route is /dynamic/[path]/static with query parameter `{}`",
                query_parameter
            ),
            None => "Route is /dynamic/[path]/static, but query parameter for `path` seems to be missing"
                .to_string(),
        }))?)
}
