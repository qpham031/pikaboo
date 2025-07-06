use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use anyhow::{Result, anyhow};
use futures::{TryFutureExt, future::join_all};
use libsql::{Connection, Database, Row, Statement, named_params};
use twilight_model::id::{
    Id,
    marker::{RoleMarker, UserMarker},
};

use crate::core::cache::EnergyData;

#[derive(Debug)]
pub struct DatabaseClient {
    database: Database,
    connection: ConnectionWrapper,
}

#[derive(Debug, Clone)]
pub struct ConnectionWrapper {
    connection: Connection,
    prep_stmts: Arc<PrepStmts>,
}

#[derive(Debug)]
pub struct CustomRole {
    pub role_id: Id<RoleMarker>,
    pub user_id: Id<UserMarker>,
    pub expires_at: u64,
    pub auto_renewal: bool,
}

type PrepStmt = Mutex<Statement>;

pub struct PrepStmts {
    sync_energy: PrepStmt,
    fetch_energy: PrepStmt,
    fetch_custom_role_by_role_id: PrepStmt,
    fetch_custom_role_by_user_id: PrepStmt,
    delete_custom_role_by_role_id: PrepStmt,
    delete_custom_role_by_user_id: PrepStmt,
}

impl Debug for PrepStmts {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl PrepStmts {
    async fn new(connection: &Connection) -> Result<PrepStmts> {
        let [
            sync_energy,
            fetch_energy,
            fetch_custom_role_by_role_id,
            fetch_custom_role_by_user_id,
            delete_custom_role_by_role_id,
            delete_custom_role_by_user_id,
        ] = join_all(
            [
                "INSERT OR REPLACE INTO energy_balance(user_id, energy) VALUES(:user_id, :energy)",
                "SELECT energy FROM energy_balance WHERE user_id = :user_id",
                "SELECT * FROM custom_roles WHERE role_id = :role_id",
                "SELECT * FROM custom_roles WHERE user_id = :user_id",
                "DELETE FROM custom_roles WHERE role_id = :role_id",
                "DELETE FROM custom_roles WHERE user_id = :user_id",
            ]
            .map(|raw| connection.prepare(raw).map_ok(Mutex::new)),
        )
        .await
        .into_iter()
        .map(Result::ok)
        .collect::<Option<Vec<PrepStmt>>>()
        .map(TryInto::<[PrepStmt; 6]>::try_into)
        .and_then(Result::ok)
        .ok_or_else(|| anyhow!("Unable to prepare statements"))?;

        Ok(PrepStmts {
            sync_energy,
            fetch_energy,
            fetch_custom_role_by_role_id,
            fetch_custom_role_by_user_id,
            delete_custom_role_by_role_id,
            delete_custom_role_by_user_id,
        })
    }
}

impl DatabaseClient {
    pub async fn new(
        url: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> Result<DatabaseClient> {
        let database =
            libsql::Builder::new_remote_replica("./local.db", url.into(), auth_token.into())
                .build()
                .await?;
        let connection = ConnectionWrapper::new(&database).await?;

        Ok(DatabaseClient {
            database,
            connection,
        })
    }
    pub fn conn(&self) -> ConnectionWrapper {
        self.connection.clone()
    }
}

impl ConnectionWrapper {
    pub async fn new(database: &Database) -> Result<ConnectionWrapper> {
        let connection = database.connect()?;

        let prep_stmts = Arc::new(PrepStmts::new(&connection).await?);
        Ok(ConnectionWrapper {
            connection,
            prep_stmts,
        })
    }

    pub async fn sync_energy_data(&self, items: Vec<EnergyData>) -> Result<()> {
        let tx = self.connection.transaction().await?;
        let mut prep_stmt = tx
            .prepare(
                "INSERT OR REPLACE INTO energy_balance(user_id, energy) VALUES(:user_id, :energy)",
            )
            .await?;

        for EnergyData {
            user_id, energy, ..
        } in items
        {
            let _ = prep_stmt
                .execute(named_params! {":user_id": user_id.get(), ":energy": energy})
                .await;
        }
        tx.commit().await?;

        Ok(())
    }
    pub async fn sync_energy_one(&self, item: EnergyData) -> Result<()> {
        let EnergyData {
            user_id,
            energy,
            is_dirty,
        } = item;

        if !is_dirty {
            return Ok(());
        }

        self.prep_stmts
            .fetch_energy
            .lock()
            .unwrap()
            .query_row(named_params! {":user_id": user_id.get(), ":energy": energy})
            .await?;

        Ok(())
    }

    pub async fn fetch_energy(&self, user_id: Id<UserMarker>) -> Result<u64> {
        let rs = self
            .prep_stmts
            .fetch_energy
            .lock()
            .unwrap()
            .query_row(named_params! {":user_id": user_id.get()})
            .await;
        let Ok(row) = rs else {
            return Ok(0);
        };

        let idx = (0..row.column_count())
            .find(|&idx| row.column_name(idx) == Some("energy"))
            .ok_or_else(|| anyhow!("Could not find column `energy`"))?;
        let energy = row.get(idx)?;
        Ok(energy)
    }

    pub async fn fetch_custom_role_by_role_id(
        &self,
        role_id: Id<RoleMarker>,
    ) -> Result<Option<CustomRole>> {
        let rs = self
            .prep_stmts
            .fetch_custom_role_by_role_id
            .lock()
            .unwrap()
            .query_row(named_params! {":role_id": role_id.get()})
            .await;

        let Ok(row) = rs else {
            return Ok(None);
        };
        Self::extract_custom_role_from_row(row)
    }

    pub async fn fetch_custom_role_by_user_id(
        &self,
        user_id: Id<UserMarker>,
    ) -> Result<Option<CustomRole>> {
        let rs = self
            .prep_stmts
            .fetch_custom_role_by_user_id
            .lock()
            .unwrap()
            .query_row(named_params! {":user_id": user_id.get()})
            .await;

        let Ok(row) = rs else {
            return Ok(None);
        };
        Self::extract_custom_role_from_row(row)
    }

    pub async fn delete_custom_role_by_role_id(&self, role_id: Id<RoleMarker>) -> Result<bool> {
        let rs = self
            .prep_stmts
            .delete_custom_role_by_role_id
            .lock()
            .unwrap()
            .execute(named_params! {":role_id": role_id.get()})
            .await?;
        Ok(rs != 0)
    }

    pub async fn delete_custom_role_by_user_id(&self, user_id: Id<UserMarker>) -> Result<bool> {
        let rs = self
            .prep_stmts
            .delete_custom_role_by_user_id
            .lock()
            .unwrap()
            .execute(named_params! {":user_id": user_id.get()})
            .await?;
        Ok(rs != 0)
    }

    fn extract_custom_role_from_row(row: Row) -> Result<Option<CustomRole>> {
        let mut role_id = None;
        let mut user_id = None;
        let mut expires_at = None;
        let mut auto_renewal = None;

        (0..row.column_count())
            .filter_map(|idx| row.column_name(idx).map(|key| (idx, key)))
            .for_each(|(idx, key)| match key {
                "role_id" => role_id = row.get(idx).ok().map(Id::new),
                "user_id" => user_id = row.get(idx).ok().map(Id::new),
                "expires_at" => expires_at = row.get(idx).ok(),
                "auto_renewal" => auto_renewal = row.get(idx).ok(),
                _ => {}
            });
        let (Some(role_id), Some(user_id), Some(expires_at), Some(auto_renewal)) =
            (role_id, user_id, expires_at, auto_renewal)
        else {
            return Err(anyhow!("Field missing"));
        };

        Ok(Some(CustomRole {
            role_id,
            user_id,
            expires_at,
            auto_renewal,
        }))
    }
}
