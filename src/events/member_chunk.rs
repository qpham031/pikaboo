use anyhow::Result;
use twilight_model::gateway::payload::incoming::MemberChunk;

use crate::core::app_state::AppState;

pub fn handle(state: AppState, member_chunk: MemberChunk) -> Result<()> {
    let mut boosters = state.cache.boosters.lock().unwrap();

    member_chunk
        .members
        .iter()
        .filter(|mem| mem.premium_since.is_some())
        .for_each(|mem| {
            boosters.insert(mem.user.id);
        });

    Ok(())
}
