use std::error::Error;

use axum::{routing::post, Router};
use handler::handle_event;

mod cmd;
mod handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    let router = Router::new().route("/", post(handle_event));

    axum::serve(listener, router).await?;

    Ok(())
}
