use http::StatusCode;
use std::error::Error;
use std::fs::read_to_string;
use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request, Response};

fn handler(_: Request) -> Result<impl IntoResponse, VercelError> {
	let text = read_to_string("./static/sample.txt").unwrap();
	let response = Response::builder()
		.status(StatusCode::OK)
		.header("Content-Type", "text/plain")
		.body(text)
		.expect("Internal Server Error");

	Ok(response)
}

fn main() -> Result<(), Box<dyn Error>> {
	Ok(lambda!(handler))
}
