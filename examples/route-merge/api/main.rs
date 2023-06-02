use vercel_runtime::{bundled_api, run, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

#[bundled_api(path = "examples/route-merge")]
pub async fn handler(req: Request) -> Result<Response<Body>, Error> {}
