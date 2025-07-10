pub mod interaction_create;
mod member_chunk;
mod member_update;
mod message_create;
mod ready;

use anyhow::Result;
use twilight_gateway::Event;

use crate::core::app_state::AppState;

pub async fn event_handler(state: AppState, event: Event) -> Result<()> {
    match event {
        Event::InteractionCreate(interaction) => {
            interaction_create::handle(state, interaction).await
        }
        Event::MessageCreate(msg) => message_create::handle(state, msg).await,
        Event::MemberUpdate(member_update) => member_update::handle(state, member_update),
        Event::MemberChunk(member_chunk) => member_chunk::handle(state, member_chunk),
        Event::Ready(ready) => ready::handle(ready),
        _ => Ok(()), // Ignore other events
    }
}
