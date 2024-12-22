use std::{env::var, sync::OnceLock};

use axum::{body::to_bytes, extract::Request, http::StatusCode};
use serde::Deserialize;

use crate::cmd;

static TRAQ_VERIFICATION_TOKEN: OnceLock<String> = OnceLock::new();

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DirectMessage {
    pub plain_text: String,
    pub user: User,
}

#[derive(Deserialize)]
struct User {
    pub id: String,
}

#[derive(Deserialize)]
struct EventDirectMessageCreated {
    pub message: DirectMessage,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    pub channel_id: String,
    pub plain_text: String,
}

#[derive(Deserialize)]
struct EventMessageCreated {
    pub message: Message,
}

pub async fn handle_event(req: Request) -> StatusCode {
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
                "DIRECT_MESSAGE_CREATED" => {
                    let body_bytes = match to_bytes(req.into_body(), usize::MAX).await {
                        Ok(body_bytes) => body_bytes,
                        Err(e) => {
                            eprintln!("{}", e);
                            return StatusCode::INTERNAL_SERVER_ERROR;
                        }
                    };
                    let event: EventDirectMessageCreated = match serde_json::from_slice(&body_bytes)
                    {
                        Ok(event) => event,
                        Err(e) => {
                            eprintln!("{}", e);
                            return StatusCode::INTERNAL_SERVER_ERROR;
                        }
                    };

                    cmd::root(event.message.plain_text, event.user.id, true).await
                }
                "MESSAGE_CREATED" => {
                    let body_bytes = match to_bytes(req.into_body(), usize::MAX).await {
                        Ok(body_bytes) => body_bytes,
                        Err(e) => {
                            eprintln!("{}", e);
                            return StatusCode::INTERNAL_SERVER_ERROR;
                        }
                    };
                    let event: EventMessageCreated = match serde_json::from_slice(&body_bytes) {
                        Ok(event) => event,
                        Err(e) => {
                            eprintln!("{}", e);
                            return StatusCode::INTERNAL_SERVER_ERROR;
                        }
                    };

                    cmd::root(event.message.plain_text, event.message.channel_id, false).await
                }
                "PING" => StatusCode::NO_CONTENT,
                _ => StatusCode::BAD_REQUEST,
            }
        }
        _ => StatusCode::BAD_REQUEST,
    }
}
