mod interaction_create;
mod member_update;
mod message_create;

use anyhow::Result;
use twilight_gateway::Event;

use crate::core::app_state::AppState;

pub async fn event_handler(event: Event, state: AppState) -> Result<()> {
    match event {
        Event::InteractionCreate(interaction) => {
            interaction_create::handle(interaction, state).await
        }
        Event::MessageCreate(msg) => message_create::handle(msg, state).await,
        Event::MemberUpdate(member_update) => member_update::handle(member_update, state).await,
        _ => Ok(()), // Ignore other events
    }
}
