use vercel_runtime::{run, Body, Error, Request, Response};

#[path = "../api/bar/baz.rs"]
mod api_bar_baz;

#[path = "../api/foo.rs"]
mod api_foo;

async fn process_request(request: Request) -> Result<Response<Body>, Error> {
    let path = request.uri().path();

    match path {
        "api/bar/baz" => api_bar_baz::handler(request).await,
        "api/foo" => api_foo::handler(request).await,
        _ => {
            unreachable!("no match")
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(process_request).await
}
