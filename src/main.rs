mod background;
mod consts;
mod core;
mod events;
mod interactions;

use std::{env, time::Duration};

use anyhow::Result;
use tracing::{error, info};
use twilight_gateway::{EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_model::gateway::payload::outgoing::RequestGuildMembers;

use crate::core::app_state::{AppState, EnvConfig};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    tracing_subscriber::fmt::init();

    info!("Load env...");
    let env = EnvConfig {
        role_scan_period: Duration::from_secs(
            env::var("ROLE_SCAN_PERIOD")
                .as_deref()
                .unwrap_or(consts::HOUR_IN_SEC_STR)
                .parse()?,
        ),
        sync_period: Duration::from_secs(
            env::var("SYNC_PERIOD")
                .as_deref()
                .unwrap_or(consts::MINUTE_IN_SEC_STR)
                .parse()?,
        ),
        owner_id: env::var("OWNER_ID")?.parse()?,
        guild_id: env::var("GUILD_ID")?.parse()?,
        discord_token: env::var("DISCORD_TOKEN")?,
        libsql_url: env::var("LIBSQL_URL")?,
        libsql_auth_token: env::var("LIBSQL_AUTH_TOKEN")?,
    };

    let app = AppState::new(env).await;
    background::run(app.clone());

    let intents = Intents::GUILD_MESSAGES | Intents::GUILD_MEMBERS;
    let mut shard = Shard::new(ShardId::ONE, app.config.env.discord_token.clone(), intents);
    let wanted_event_types = EventTypeFlags::READY
        | EventTypeFlags::MESSAGE_CREATE
        | EventTypeFlags::MEMBER_CHUNK
        | EventTypeFlags::MEMBER_UPDATE;

    let request = RequestGuildMembers::builder(app.config.env.guild_id).query("", None);
    shard.command(&request);

    while let Some(item) = shard.next_event(wanted_event_types).await {
        let Ok(event) = item else {
            error!(source = ?item.unwrap_err(), "Error receiving event");
            continue;
        };

        // Not from the Discord system or the targetted server
        if event
            .guild_id()
            .is_some_and(|guild_id| guild_id != app.config.env.guild_id)
        {
            continue;
        }

        let app = app.clone();
        tokio::spawn(async move {
            let Err(err) = events::event_handler(app, event).await else {
                return;
            };
            error!(?err, "Error handling event");
        });
    }

    Ok(())
}
