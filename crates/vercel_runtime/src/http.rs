use lambda_http::{http::StatusCode, Body, Error, Response};
use serde::Serialize;

pub fn ok(val: impl Serialize) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::Text(serde_json::to_string(&val).unwrap()))?)
}

pub fn bad_request(val: impl Serialize) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("content-type", "application/json")
        .body(Body::Text(serde_json::to_string(&val).unwrap()))?)
}

pub fn not_found(val: impl Serialize) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "application/json")
        .body(Body::Text(serde_json::to_string(&val).unwrap()))?)
}

pub fn unauthorized(val: impl Serialize) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("content-type", "application/json")
        .body(Body::Text(serde_json::to_string(&val).unwrap()))?)
}

pub fn internal_server_error(val: impl Serialize) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("content-type", "application/json")
        .body(Body::Text(serde_json::to_string(&val).unwrap()))?)
}
