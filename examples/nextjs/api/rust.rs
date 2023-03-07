use serde_json::json;
use std::time::Instant;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let start = Instant::now();

    let seed = 42;
    let mut rng = oorandom::Rand32::new(seed);

    const RADIUS: u64 = 424242;
    const LOOPS: u64 = 1_000_000;

    let mut counter = 0;

    for _ in 0..LOOPS {
        let x: u64 = rng.rand_range(1..RADIUS as u32).into();
        let y: u64 = rng.rand_range(1..RADIUS as u32).into();

        if (x.pow(2) + y.pow(2)) < (RADIUS.pow(2)) {
            counter += 1;
        }
    }

    let pi = (4.0 * counter as f64) / LOOPS as f64;

    let duration = start.elapsed();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
                "runtime": "rust",
                "message": format!("{} / {}", counter, LOOPS),
                "time": format!("{:.2?}", duration),
                "pi": pi
            })
            .into(),
        )?;

    Ok(response)
}
