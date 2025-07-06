use anyhow::Result;
use twilight_model::gateway::payload::incoming::MemberUpdate;

use crate::core::app_state::AppState;

pub async fn handle(member_update: Box<MemberUpdate>, state: AppState) -> Result<()> {
    todo!()
}
