use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use anyhow::{Result, anyhow};
use libsql::{Connection, Database, Value, named_params};
use serde::Deserialize;
use tracing::warn;
use twilight_mention::{
    Mention,
    timestamp::{Timestamp, TimestampStyle},
};
use twilight_model::{
    id::{
        Id,
        marker::{RoleMarker, UserMarker},
    },
    util::ImageHash,
};
use twilight_util::snowflake::Snowflake;

use crate::core::{
    cache::EnergyData,
    config::{ConfigInner, ConfigWrapperBuilder},
};

#[derive(Debug)]
pub struct DatabaseClient {
    database: Database,
    connection: ConnectionWrapper,
}

#[derive(Debug, Clone)]
pub struct ConnectionWrapper(Connection);

#[derive(Debug, Clone, Deserialize)]
pub struct CustomRole {
    pub role_id: Id<RoleMarker>,
    pub user_id: Id<UserMarker>,
    pub auto_renewal: bool,
    pub expires_at: Option<u64>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub color: u32,
    #[serde(default)]
    pub icon: RoleIcon,
    #[serde(default)]
    pub mentionable: bool,
}

impl Display for CustomRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const EXP_MAX: u64 = 8640000000000;
        const TIMESTAMP_STYLE: Option<TimestampStyle> = Some(TimestampStyle::ShortDate);

        let create_at = self.role_id.timestamp() as u64 / 1000;
        let expires_at = self.expires_at.unwrap_or(EXP_MAX);

        write!(
            f,
            "**Role:** {role}\n\
            **Owner:** {owner}\n\
            **Color:** `#{color:06X}`\n\
            **Icon:** {icon}\n\
            **Mentionable:** {mentionable}\n\
            \n\
            **Created on:** {created_at}\n\
            **Expires on:** {expires_at}\n\
            **Auto-renewal:** `{auto_renewal}`",
            role = self.role_id.mention(),
            owner = self.user_id.mention(),
            auto_renewal = self.auto_renewal,
            created_at = Timestamp::new(create_at, TIMESTAMP_STYLE).mention(),
            expires_at = Timestamp::new(expires_at, TIMESTAMP_STYLE).mention(),
            color = self.color,
            icon = self.icon,
            mentionable = self.mentionable
        )
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub enum RoleIcon {
    Custom(ImageHash),
    Unicode(String),
    #[default]
    None,
}

impl Display for RoleIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoleIcon::Custom(hash) => write!(f, "Custom({hash})"),
            RoleIcon::Unicode(emoji) => write!(f, "Unicode({emoji})"),
            RoleIcon::None => write!(f, "None"),
        }
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
        let connection = ConnectionWrapper(database.connect()?);

        Ok(DatabaseClient {
            database,
            connection,
        })
    }

    pub fn clone_conn(&self) -> ConnectionWrapper {
        self.connection.clone()
    }

    pub async fn sync(&self) -> Result<()> {
        self.database.sync().await?;
        Ok(())
    }
}

impl Deref for DatabaseClient {
    type Target = ConnectionWrapper;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl ConnectionWrapper {
    pub async fn new(database: &Database) -> Result<ConnectionWrapper> {
        let connection = database.connect()?;
        Ok(ConnectionWrapper(connection))
    }

    pub async fn sync_energy_data(&self, items: Vec<EnergyData>) -> Result<()> {
        let tx = self.0.transaction().await?;
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

        self.0
            .query(
                "INSERT OR REPLACE INTO energy_balance(user_id, energy) VALUES(:user_id, :energy)",
                named_params! {":user_id": user_id.get(), ":energy": energy},
            )
            .await?;

        Ok(())
    }

    pub async fn fetch_energy(&self, user_id: Id<UserMarker>) -> Result<u64> {
        let rs = self
            .0
            .query(
                "SELECT energy FROM energy_balance WHERE user_id = :user_id",
                named_params! {":user_id": user_id.get()},
            )
            .await?
            .next()
            .await?;
        let Some(row) = rs else {
            return Ok(0);
        };

        let idx = (0..row.column_count())
            .find(|&idx| row.column_name(idx) == Some("energy"))
            .ok_or_else(|| anyhow!("Could not find column `energy`"))?;
        let energy = row.get(idx)?;
        Ok(energy)
    }

    pub async fn consume_energy(&self, user_id: Id<UserMarker>, amount: u64) -> Result<bool> {
        let changes = self
            .0
            .execute(
                "UPDATE energy_balance SET energy = energy - :amount WHERE user_id = :user_id AND energy >= :amount",
                named_params! {":user_id": user_id.get(), ":amount": amount},
            )
            .await?;
        Ok(changes != 0)
    }

    pub async fn fetch_custom_roles(&self) -> Result<Vec<CustomRole>> {
        let mut rows = self.0.query("SELECT * FROM custom_roles", ()).await?;
        let mut collection = vec![];

        loop {
            let rs = rows.next().await;
            let Ok(row_op) = rs else {
                warn!(source = ?rs.unwrap_err(), "Unable to get CustomRole row");
                continue;
            };
            let Some(row) = row_op else {
                break;
            };
            let data_rs = libsql::de::from_row(&row);
            let Ok(data) = data_rs else {
                warn!(source = ?data_rs.unwrap_err(), "Unable to parse role");
                continue;
            };
            collection.push(data);
        }

        Ok(collection)
    }

    pub async fn fetch_custom_role_by_role_id(
        &self,
        role_id: Id<RoleMarker>,
    ) -> Result<Option<CustomRole>> {
        let rs = self
            .0
            .query(
                "SELECT * FROM custom_roles WHERE role_id = :role_id",
                named_params! {":role_id": role_id.get()},
            )
            .await?
            .next()
            .await?;

        let Some(row) = rs else {
            return Ok(None);
        };

        Ok(Some(libsql::de::from_row(&row)?))
    }

    pub async fn fetch_custom_role_by_user_id(
        &self,
        user_id: Id<UserMarker>,
    ) -> Result<Option<CustomRole>> {
        let rs = self
            .0
            .query(
                "SELECT * FROM custom_roles WHERE user_id = :user_id",
                named_params! {":user_id": user_id.get()},
            )
            .await?
            .next()
            .await?;

        let Some(row) = rs else {
            return Ok(None);
        };

        Ok(Some(libsql::de::from_row(&row)?))
    }

    pub async fn delete_custom_role_by_role_id(&self, role_id: Id<RoleMarker>) -> Result<bool> {
        let affected_rows = self
            .0
            .execute(
                "DELETE FROM custom_roles WHERE role_id = :role_id",
                named_params! {":role_id": role_id.get()},
            )
            .await?;
        Ok(affected_rows != 0)
    }

    pub async fn delete_custom_role_by_user_id(&self, user_id: Id<UserMarker>) -> Result<bool> {
        let affected_rows = self
            .0
            .execute(
                "DELETE FROM custom_roles WHERE user_id = :user_id",
                named_params! {":user_id": user_id.get()},
            )
            .await?;
        Ok(affected_rows != 0)
    }

    pub async fn update_custom_role(&self, role: &CustomRole) -> Result<bool> {
        let affected_rows = self
            .0
            .execute(
                "INSERT OR REPLACE INTO custom_roles (role_id, user_id, expires_at, auto_renewal) VALUES (:role_id, :user_id, :expires_at, :auto_renewal)",
                named_params! {
                    ":role_id": role.user_id.get(),
                    ":user_id": role.user_id.get(),
                    ":expires_at": role.expires_at.map(Value::try_from).unwrap_or(Ok(Value::Null))?,
                    ":auto_renewal": role.auto_renewal,
                },
            )
            .await?;
        Ok(affected_rows != 0)
    }

    pub async fn fetch_config(&self) -> Result<ConfigInner> {
        let mut rows = self.0.query("SELECT * FROM app_config", ()).await?;
        let mut builder = ConfigWrapperBuilder::default();

        loop {
            let row_rs = rows.next().await;
            let Ok(row_op) = row_rs else {
                warn!(source = ?row_rs.unwrap_err(), "Failed to get the next Config row");
                continue;
            };
            let Some(row) = row_op else {
                break;
            };
            let (Ok(key), Ok(value)) = (row.get_str(0), row.get_str(1)) else {
                continue;
            };
            builder.set_field(key, value);
        }

        builder.try_build()
    }
}
