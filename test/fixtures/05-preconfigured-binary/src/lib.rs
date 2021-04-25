use http::StatusCode;
use vercel_lambda::{error::VercelError, IntoResponse, Response};

pub fn say_hello() -> Result<impl IntoResponse, VercelError> {
  let response = Response::builder()
    .status(StatusCode::OK)
    .header("Content-Type", "text/plain")
    .body("Hello, world!")?;

  Ok(response)
}
