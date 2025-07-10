use anyhow::Result;
use twilight_model::gateway::payload::incoming::MemberUpdate;

use crate::core::app_state::AppState;

pub fn handle(state: AppState, member_update: Box<MemberUpdate>) -> Result<()> {
    let is_booster = member_update.premium_since.is_some();
    let mut boosters = state.cache.boosters.lock().unwrap();

    if is_booster {
        boosters.insert(member_update.user.id);
    } else {
        boosters.remove(&member_update.user.id);
    }

    Ok(())
}
