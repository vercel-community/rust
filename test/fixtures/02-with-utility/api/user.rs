use util::return_foo;
use http::{StatusCode};
use now_lambda::{error::NowError, lambda, IntoResponse, Request, Response};
use std::error::Error;

fn handler(_: Request) -> Result<impl IntoResponse, NowError> {
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
