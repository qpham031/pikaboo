use anyhow::Result;
use tracing::info;
use twilight_model::gateway::payload::incoming::Ready;

pub fn handle(ready: Box<Ready>) -> Result<()> {
    info!("{} is ready!", ready.user.name);
    Ok(())
}
