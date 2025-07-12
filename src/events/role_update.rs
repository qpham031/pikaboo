use anyhow::Result;
use twilight_model::gateway::payload::incoming::RoleUpdate;

use crate::core::app_state::AppState;

pub fn handle(state: AppState, role_update: RoleUpdate) -> Result<()> {
    Ok(())
}
