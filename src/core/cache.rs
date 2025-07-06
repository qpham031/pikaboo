use std::{collections::HashSet, sync::Mutex};

use anyhow::Result;
use lru::LruCache;
use twilight_model::id::{
    Id,
    marker::{RoleMarker, UserMarker},
};

use crate::core::database::ConnectionWrapper;

#[derive(Debug)]
pub struct Cache {
    pub energy_balance: EnergyBalance,
    pub roles: HashSet<Id<RoleMarker>>,
}

impl Cache {
    pub fn new(conn: ConnectionWrapper) -> Cache {
        Cache {
            energy_balance: EnergyBalance::new(50, conn),
            roles: Default::default(),
        }
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
