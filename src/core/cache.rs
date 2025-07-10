use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Mutex,
};

use anyhow::Result;
use lru::LruCache;
use twilight_model::id::{Id, marker::UserMarker};

use crate::core::database::{ConnectionWrapper, CustomRole};

#[derive(Debug)]
pub struct Cache {
    pub energy_balance: EnergyBalance,
    pub user_custom_roles: UserCustomRole,
    pub boosters: Mutex<HashSet<Id<UserMarker>>>,
}

impl Cache {
    pub async fn new(conn: ConnectionWrapper) -> Result<Cache> {
        Ok(Cache {
            energy_balance: EnergyBalance::new(50, conn.clone()),
            user_custom_roles: UserCustomRole::new(conn).await?,
            boosters: Default::default(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnergyData {
    pub user_id: Id<UserMarker>,
    pub energy: u64,
    pub is_dirty: bool,
}

#[derive(Debug)]
pub struct EnergyBalance {
    connection: ConnectionWrapper,
    balance: Mutex<LruCache<Id<UserMarker>, EnergyData>>,
}

impl EnergyBalance {
    fn new(cap: usize, connection: ConnectionWrapper) -> EnergyBalance {
        EnergyBalance {
            balance: Mutex::new(LruCache::new(cap.try_into().unwrap())),
            connection,
        }
    }

    async fn fetch(&self, user_id: Id<UserMarker>) -> Result<u64> {
        let energy = self.connection.fetch_energy(user_id).await?;
        let data = EnergyData {
            user_id,
            energy,
            is_dirty: false,
        };

        let evicted_data = self.balance.lock().unwrap().push(user_id, data);
        if let Some((_, data)) = evicted_data {
            self.connection.sync_energy_one(data).await?;
        }

        Ok(energy)
    }

    pub async fn get(&self, user_id: Id<UserMarker>) -> Result<u64> {
        let data = self.balance.lock().unwrap().get(&user_id).cloned();
        let energy = match data {
            Some(data) => data.energy,
            None => self.fetch(user_id).await?,
        };

        Ok(energy)
    }

    pub async fn add_one(&self, user_id: Id<UserMarker>) -> Result<()> {
        loop {
            let cache_hit = self
                .balance
                .lock()
                .unwrap()
                .get_mut(&user_id)
                .map(|data| data.energy += 1)
                .is_some();

            if cache_hit {
                return Ok(());
            }

            self.fetch(user_id).await?;
        }
    }

    pub async fn consume_energy(&self, user_id: Id<UserMarker>, amount: u64) -> Result<bool> {
        let status = self.balance.lock().unwrap().get_mut(&user_id).map(|data| {
            if data.energy > amount {
                data.energy -= amount;
                data.is_dirty = true;
                return true;
            }
            false
        });

        if let Some(status) = status {
            return Ok(status);
        }

        let status = self.connection.consume_energy(user_id, amount).await?;
        Ok(status)
    }

    pub async fn sync_energy_data(&self) -> Result<()> {
        let dirty_data = self
            .balance
            .lock()
            .unwrap()
            .iter_mut()
            .filter(|(_, data)| data.is_dirty)
            .map(|(_, data)| {
                data.is_dirty = false;
                *data
            })
            .collect::<Vec<_>>();

        self.connection.sync_energy_data(dirty_data).await
    }
}

#[derive(Debug, Default)]
pub struct UserCustomRole(Mutex<HashMap<Id<UserMarker>, CustomRole>>);

impl UserCustomRole {
    pub async fn new(conn: ConnectionWrapper) -> Result<UserCustomRole> {
        let collection = conn.fetch_custom_roles().await?;
        let map = collection
            .into_iter()
            .map(|role| (role.user_id, role))
            .collect();
        Ok(UserCustomRole(Mutex::new(map)))
    }
    pub fn get(&self, user_id: Id<UserMarker>) -> Option<CustomRole> {
        self.0.lock().unwrap().get(&user_id).cloned()
    }
    pub fn remove(&self, user_id: Id<UserMarker>) -> Option<CustomRole> {
        self.0.lock().unwrap().remove(&user_id)
    }
    pub fn update(&self, role: CustomRole) {
        self.0.lock().unwrap().insert(role.user_id, role);
    }
}

impl Deref for UserCustomRole {
    type Target = Mutex<HashMap<Id<UserMarker>, CustomRole>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
