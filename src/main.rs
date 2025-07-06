mod core;
mod events;
mod interactions;

use anyhow::Result;
use tracing::{error, info};
use twilight_gateway::{EventTypeFlags, Intents, Shard, ShardId, StreamExt};

use crate::core::app_state::{AppState, Config};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    tracing_subscriber::fmt::init();

    info!("Load Config...");
    let config = Config {
        discord_token: std::env::var("DISCORD_TOKEN")?,
        libsql_url: std::env::var("LIBSQL_URL")?,
        libsql_auth_token: std::env::var("LIBSQL_AUTH_TOKEN")?,
        owner_id: std::env::var("OWNER_ID")?.parse()?,
    };

    let app = AppState::new(config).await;
    let intents = Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::GUILD_MEMBERS;
    let mut shard = Shard::new(ShardId::ONE, app.config.discord_token.clone(), intents);
    let wanted_event_types = EventTypeFlags::READY
        | EventTypeFlags::GUILDS
        | EventTypeFlags::MEMBER_UPDATE
        | EventTypeFlags::MEMBER_REMOVE;

    while let Some(item) = shard.next_event(wanted_event_types).await {
        let Ok(event) = item else {
            error!(source = ?item.unwrap_err(), "Error receiving event");
            continue;
        };

        let app = app.clone();
        tokio::spawn(async move {
            let Err(err) = events::event_handler(event, app).await else {
                return;
            };
            error!(?err, "Error handling event");
        });
    }

    Ok(())
}
