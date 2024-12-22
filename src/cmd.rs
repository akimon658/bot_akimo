use std::sync::OnceLock;

use axum::http::StatusCode;
use clap::{Parser, Subcommand};
use traq::{
    apis::{configuration::Configuration, message_api, user_api},
    models::PostMessageRequest,
};

#[derive(Parser)]
struct Command {
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    Ping,
}

static TRAQ_ACCESS_TOKEN: OnceLock<String> = OnceLock::new();
static TRAQ_CONFIG: OnceLock<Configuration> = OnceLock::new();

pub async fn root(text: String, target_id: String, is_dm: bool) -> StatusCode {
    let config = TRAQ_CONFIG.get_or_init(|| {
        let access_token = TRAQ_ACCESS_TOKEN.get_or_init(|| {
            std::env::var("TRAQ_ACCESS_TOKEN").expect("TRAQ_ACCESS_TOKEN is not set")
        });

        Configuration {
            bearer_access_token: Some(access_token.to_string()),
            ..Default::default()
        }
    });
    let cmd = Command::parse_from(text.split_whitespace());

    match cmd.subcommand {
        Subcommands::Ping => {
            if is_dm {
                match user_api::post_direct_message(
                    &config,
                    &target_id,
                    Some(PostMessageRequest {
                        content: "https://q.trap.jp/files/0193eee8-5abd-7c98-bee8-de164bf705e9"
                            .to_string(),
                        embed: None,
                    }),
                )
                .await
                {
                    Ok(_) => StatusCode::NO_CONTENT,
                    Err(e) => {
                        eprintln!("Failed to send a message to user {}: {}", target_id, e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                };
            } else {
                match message_api::post_message(
                    &config,
                    &target_id,
                    Some(PostMessageRequest {
                        content: "https://q.trap.jp/files/0193ecfb-fb3b-7a3c-8547-377733691e13"
                            .to_string(),
                        embed: None,
                    }),
                )
                .await
                {
                    Ok(_) => StatusCode::NO_CONTENT,
                    Err(e) => {
                        eprintln!("Failed to send a message to channel {}: {}", target_id, e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                };
            }
        }
    }

    StatusCode::NO_CONTENT
}
