use http::StatusCode;
use std::error::Error;
use util::return_foo;
use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request, Response};

fn handler(_: Request) -> Result<impl IntoResponse, VercelError> {
	let response = Response::builder()
		.status(StatusCode::OK)
		.header("Content-Type", "text/plain")
		.body(return_foo())
		.expect("Internal Server Error");

	Ok(response)
}

fn main() -> Result<(), Box<dyn Error>> {
	Ok(lambda!(handler))
}
