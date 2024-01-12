use tracing::Level;
use crate::config::{Config, LoggingLevel};
use crate::error::Error;

mod playback;
mod api;
mod util;
mod config;
mod error;
mod ext;
mod channel;
mod abort;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = util::deserialize_yaml::<_, Config>(std::fs::File::open("nightingale.yml")?)?;

    if config.logging.enable {
        tracing_subscriber::fmt()
            .with_max_level(<LoggingLevel as Into<Level>>::into(config.logging.level))
            .init();
    }

    api::start_http(config).await?;
    Ok(())
}
