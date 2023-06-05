use std::collections::HashMap;

use url::Url;
use vercel_runtime::{Body, Error, Request, Response, StatusCode};

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let parsed_url = Url::parse(&req.uri().to_string()).unwrap();
    let hash_query: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    let path_query_parameter = hash_query.get("path");
    let id_query_parameter = hash_query.get("id");

    if path_query_parameter.is_none() || id_query_parameter.is_none() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(Body::Text(
                "Route is /dynamic/[path]/[id], but query parameters seems to be missing"
                    .to_string(),
            ))?);
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::Text(format!(
            "Route is /dynamic/[path]/[id] with `path` query parameter `{}` and `id` query parameter `{}`",
            path_query_parameter.unwrap(),
            id_query_parameter.unwrap()
        )))?)
}
