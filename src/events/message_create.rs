use anyhow::Result;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::core::app_state::AppState;

pub async fn handle(msg: Box<MessageCreate>, state: AppState) -> Result<()> {
    todo!()
}
