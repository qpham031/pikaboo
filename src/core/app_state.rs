use std::{
    collections::HashSet,
    ops::Deref,
    sync::{Arc, Mutex},
};

use tracing::info;
use twilight_http::Client as HttpClient;
use twilight_model::id::{Id, marker::UserMarker};

use crate::core::{cache::Cache, database::DatabaseClient};

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub libsql_url: String,
    pub libsql_auth_token: String,
    pub owner_id: Id<UserMarker>,
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
    pub async fn new(config: Config) -> AppState {
        info!("Initializing AppState contents...");

        let app = HttpClient::new(config.discord_token.clone());
        info!("HTTP client initialized.");

        let db = DatabaseClient::new(&config.libsql_url, &config.libsql_auth_token)
            .await
            .expect("Failed to connect to database");
        info!("Database client initialized.");

        let cache = Cache::new(db.conn());
        info!("Cache initialized.");

        let checkin_note = CheckinNote::default();
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

#[derive(Debug, Default)]
pub struct CheckinNote(Mutex<HashSet<Id<UserMarker>>>);

impl CheckinNote {
    pub fn checkin(&self, user_id: Id<UserMarker>) -> bool {
        self.0.lock().unwrap().insert(user_id)
    }
    pub fn reset(&self) {
        self.0.lock().unwrap().clear();
    }
}
