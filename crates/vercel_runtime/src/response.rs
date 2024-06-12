use lambda_http::http::{
    header::{HeaderMap, HeaderValue},
    Response,
};
use lambda_http::Body;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventResponse {
    pub status_code: u16,
    #[serde(
        skip_serializing_if = "HeaderMap::is_empty",
        with = "http_serde::header_map"
    )]
    pub headers: HeaderMap<HeaderValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

impl Default for EventResponse {
    fn default() -> Self {
        Self {
            status_code: 200,
            headers: Default::default(),
            body: Default::default(),
            encoding: Default::default(),
        }
    }
}

impl<T> From<Response<T>> for EventResponse
where
    T: Into<Body>,
{
    fn from(value: Response<T>) -> Self {
        let (parts, bod) = value.into_parts();
        let (encoding, body) = match bod.into() {
            Body::Empty => (None, None),
            b @ Body::Text(_) => (None, Some(b)),
            b @ Body::Binary(_) => (Some("base64".to_string()), Some(b)),
        };
        EventResponse {
            status_code: parts.status.as_u16(),
            body,
            headers: parts.headers,
            encoding,
        }
    }
}
