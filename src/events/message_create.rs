use anyhow::Result;
use twilight_model::gateway::payload::incoming::MessageCreate;
use twilight_util::snowflake::Snowflake;

use crate::core::app_state::AppState;

pub async fn handle(state: AppState, msg: Box<MessageCreate>) -> Result<()> {
    let timestamp = msg.id.timestamp() as u64 / 1000;
    let user_id = msg.author.id;
    let channel_id = msg.channel_id;
    let in_the_zone = state.config.read().unwrap().zones.contains(&channel_id);

    if !in_the_zone {
        return Ok(());
    }

    let new_checkin = state.checkin_note.checkin(user_id, timestamp);

    if !new_checkin {
        return Ok(());
    }

    state.cache.energy_balance.add_one(user_id).await?;

    Ok(())
}
