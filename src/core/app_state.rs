use std::{
    collections::HashSet,
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use tracing::info;
use twilight_http::Client as HttpClient;
use twilight_model::id::{
    Id,
    marker::{GuildMarker, UserMarker},
};

use crate::core::{
    cache::Cache,
    config::{Config, ConfigInner},
    database::DatabaseClient,
};

#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub role_scan_period: Duration,
    pub sync_period: Duration,
    pub owner_id: Id<UserMarker>,
    pub guild_id: Id<GuildMarker>,
    pub discord_token: String,
    pub libsql_url: String,
    pub libsql_auth_token: String,
}

#[derive(Debug)]
pub struct AppStateInner {
    pub app: HttpClient,
    pub config: Config,
    pub db: DatabaseClient,
    pub checkin_note: CheckinNote,
    pub cache: Cache,
}

#[derive(Debug, Clone)]
pub struct AppState(Arc<AppStateInner>);

impl AppState {
    pub async fn new(env: EnvConfig) -> AppState {
        info!("Initializing AppState contents...");

        let app = HttpClient::new(env.discord_token.clone());
        info!("HTTP client initialized.");

        let db = DatabaseClient::new(&env.libsql_url, &env.libsql_auth_token)
            .await
            .expect("Failed to connect to database");
        info!("Database client initialized.");

        let cache = Cache::new(db.clone_conn())
            .await
            .expect("Failed to initialize cache");
        info!("Cache initialized.");

        let config = Config::new(db.clone_conn(), env)
            .await
            .expect("Failed fetching Config");
        info!("Config initialized.");

        let checkin_note = CheckinNote::new(config.inner.clone());
        info!("Check-in note initialized.");
        AppState(Arc::new(AppStateInner {
            app,
            config,
            db,
            cache,
            checkin_note,
        }))
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct CheckinNote {
    config: Arc<RwLock<ConfigInner>>,
    inner: Mutex<CheckinNoteInner>,
}

#[derive(Debug, Default)]
pub struct CheckinNoteInner {
    last_timestamp: u64,
    notes: HashSet<Id<UserMarker>>,
}

impl CheckinNote {
    pub fn new(config: Arc<RwLock<ConfigInner>>) -> CheckinNote {
        CheckinNote {
            config,
            inner: Default::default(),
        }
    }

    pub fn checkin(&self, user_id: Id<UserMarker>, timestamp: u64) -> bool {
        let cd = self.config.read().unwrap().cooldown;
        let mut inner = self.inner.lock().unwrap();

        if inner.last_timestamp + cd < timestamp {
            inner.notes.clear();
        }

        inner.notes.insert(user_id)
    }
}
