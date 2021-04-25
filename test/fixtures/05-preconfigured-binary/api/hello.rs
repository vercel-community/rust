use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request};

use preconfigured_binary::say_hello;

fn main() {
  lambda!(handler);
}

fn handler(_request: Request) -> Result<impl IntoResponse, VercelError> {
  say_hello()
}
