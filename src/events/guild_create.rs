use std::collections::HashMap;

use anyhow::Result;
use twilight_model::{gateway::payload::incoming::GuildCreate, guild::Guild};

use crate::core::{app_state::AppState, database::RoleIcon};

pub fn handle(state: AppState, guild_create: Box<GuildCreate>) -> Result<()> {
    let GuildCreate::Available(guild) = *guild_create else {
        return Ok(());
    };

    let Guild { members, roles, .. } = guild;

    // Update cache boosters
    let boosters = members
        .iter()
        .filter(|mem| mem.premium_since.is_some())
        .map(|mem| mem.user.id)
        .collect();

    *state.cache.boosters.lock().unwrap() = boosters;

    // Update cache user custom roles
    let mut user_custom_roles = state.cache.user_custom_roles.lock().unwrap();
    let mut custom_roles: HashMap<_, _> = user_custom_roles
        .iter_mut()
        .map(|(_, role)| (role.role_id, role))
        .collect();

    for role in roles {
        let Some(crole) = custom_roles.get_mut(&role.id) else {
            continue;
        };
        crole.name = role.name;
        crole.color = role.color;
        crole.mentionable = role.mentionable;
        crole.icon = role
            .icon
            .map(RoleIcon::Custom)
            .or(role.unicode_emoji.map(RoleIcon::Unicode))
            .unwrap_or_default();
    }
    drop(user_custom_roles);

    Ok(())
}
