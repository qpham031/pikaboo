use std::fmt::Write;

use anyhow::{Result, anyhow};
use tokio::join;
use tracing::error;

use crate::{
    consts::MONTH_IN_SEC,
    core::{app_state::AppState, database::CustomRole},
};

pub fn run(state: AppState) {
    tokio::spawn(sync_energy(state.clone()));
    tokio::spawn(scan_custom_roles(state.clone()));
}

async fn sync_energy(state: AppState) {
    loop {
        tokio::time::sleep(state.config.env.sync_period).await;
        if let Err(err) = state.cache.energy_balance.sync_energy_data().await {
            error!("Unable to sync energy: {err}")
        }
    }
}

async fn scan_custom_roles(state: AppState) {
    async fn renew_role(
        state: AppState,
        mut role: CustomRole,
        renew_fee: u64,
        now: u64,
    ) -> Result<()> {
        let done = state
            .cache
            .energy_balance
            .consume_energy(role.user_id, renew_fee)
            .await?;

        if !done {
            remove_role(state, role).await?;
            return Ok(());
        }

        role.expires_at.replace(now + MONTH_IN_SEC);
        state.db.update_custom_role(role).await?;
        state.cache.user_custom_roles.update(role);
        Ok(())
    }

    async fn remove_role(state: AppState, role: CustomRole) -> Result<()> {
        let guild_id = state.config.env.guild_id;

        let (discord_rs, database_rs) = join!(
            state.app.delete_role(guild_id, role.role_id),
            state.db.delete_custom_role_by_role_id(role.role_id)
        );

        // Throw error message
        if discord_rs.is_err() || database_rs.is_err() {
            let mut msg = format!("Fail to remove role <{}>:\n", role.role_id);

            if let Err(err) = discord_rs {
                let _ = writeln!(&mut msg, "- Discord: {err:?}");
            }

            if let Err(err) = database_rs {
                let _ = writeln!(&mut msg, "- Database: {err:?}");
            }

            return Err(anyhow!(msg));
        }

        Ok(())
    }

    loop {
        let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        let expired_roles = {
            let mut user_custom_roles = state.cache.user_custom_roles.lock().unwrap();
            let boosters = state.cache.boosters.lock().unwrap();

            user_custom_roles
                .extract_if(|_, crole| {
                    crole.expires_at.is_some_and(|lifetime| lifetime < now)
                        && !boosters.contains(&crole.user_id)
                })
                .map(|(_key, value)| value)
                .collect::<Vec<_>>()
        };
        let renew_fee = state.config.inner.read().unwrap().service_fee.custom_role;

        for role in expired_roles {
            let state = state.clone();

            let rs = if role.auto_renewal {
                renew_role(state, role, renew_fee, now).await
            } else {
                remove_role(state, role).await
            };

            if let Err(err) = rs {
                error!("Unable to remove/renew role <{}>: {err}", role.role_id);
            }
        }

        tokio::time::sleep(state.config.env.role_scan_period).await;
    }
}
