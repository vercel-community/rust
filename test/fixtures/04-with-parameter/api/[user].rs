use http::StatusCode;
use std::error::Error;
use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request, Response};

fn handler(request: Request) -> Result<impl IntoResponse, VercelError> {
	let response = Response::builder()
		.status(StatusCode::OK)
		.header("Content-Type", "text/plain")
		.body(String::from(request.uri().query().unwrap()))
		.expect("Internal Server Error");

	Ok(response)
}

fn main() -> Result<(), Box<dyn Error>> {
	Ok(lambda!(handler))
}
