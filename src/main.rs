use std::{env::var, error::Error, sync::OnceLock};

use axum::{extract::Request, http::StatusCode, routing::post, Router};

static TRAQ_VERIFICATION_TOKEN: OnceLock<String> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    let router = Router::new().route("/", post(handle_event));

    axum::serve(listener, router).await?;

    Ok(())
}

async fn handle_event(req: Request) -> StatusCode {
    let headers = req.headers();
    let request_token = match headers.get("X-TRAQ-BOT-TOKEN") {
        Some(token) => token,
        None => return StatusCode::BAD_REQUEST,
    };
    let verification_token = TRAQ_VERIFICATION_TOKEN.get_or_init(|| {
        var("TRAQ_VERIFICATION_TOKEN").expect("TRAQ_VERIFICATION_TOKEN is not set")
    });

    if request_token != verification_token {
        return StatusCode::FORBIDDEN;
    }

    match headers.get("X-TRAQ-BOT-EVENT") {
        Some(event) => {
            let event_str = match event.to_str() {
                Ok(event_str) => event_str,
                Err(e) => {
                    eprintln!("{}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR;
                }
            };

            match event_str {
                "PING" => StatusCode::OK,
                _ => StatusCode::BAD_REQUEST,
            }
        }
        _ => StatusCode::BAD_REQUEST,
    }
}
