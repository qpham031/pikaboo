use std::{
    collections::HashSet,
    ops::Deref,
    sync::{Arc, RwLock},
};

use anyhow::{Result, anyhow};
use twilight_model::id::{Id, marker::ChannelMarker};

use crate::core::{app_state::EnvConfig, database::ConnectionWrapper};

#[derive(Debug)]
pub struct Config {
    conn: ConnectionWrapper,
    pub inner: Arc<RwLock<ConfigInner>>,
    pub env: EnvConfig,
}

#[derive(Debug, Clone)]
pub struct ConfigInner {
    pub cooldown: u64,
    pub service_fee: ServiceFee,
    pub zones: HashSet<Id<ChannelMarker>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ServiceFee {
    pub custom_role: u64,
    pub nickname: u64,
}

impl Config {
    pub async fn new(conn: ConnectionWrapper, env: EnvConfig) -> Result<Config> {
        let inner = Arc::new(RwLock::new(conn.fetch_config().await?));

        Ok(Config { conn, inner, env })
    }
}

impl Deref for Config {
    type Target = RwLock<ConfigInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Default)]
pub struct ConfigWrapperBuilder {
    pub cooldown: Option<u64>,
    pub custom_role_fee: Option<u64>,
    pub nickname_fee: Option<u64>,
    pub zones: Option<HashSet<Id<ChannelMarker>>>,
}

impl ConfigWrapperBuilder {
    pub fn set_field(&mut self, field: &str, value: &str) {
        match field {
            "cooldown" => self.cooldown = value.parse().ok(),
            "custom_role_fee" => self.custom_role_fee = value.parse().ok(),
            "nickname_fee" => self.nickname_fee = value.parse().ok(),
            "zones" => self.zones = serde_json::from_str(value).ok(),
            _ => {}
        };
    }

    pub fn try_build(self) -> Result<ConfigInner> {
        fn inner(this: ConfigWrapperBuilder) -> Option<ConfigInner> {
            Some(ConfigInner {
                cooldown: this.cooldown?,
                service_fee: ServiceFee {
                    custom_role: this.custom_role_fee?,
                    nickname: this.nickname_fee?,
                },
                zones: this.zones?,
            })
        }
        inner(self).ok_or_else(|| anyhow!("Config could not be built"))
    }
}
