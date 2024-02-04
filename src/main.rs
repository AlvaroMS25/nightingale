use std::time::Duration;
use tracing::{info, Level};
use crate::config::{Config, LoggingLevel};
use crate::error::Error;

mod playback;
mod api;
mod config;
mod error;
mod channel;
mod abort;
mod metrics;
mod search;
mod ext;

const NIGHTINGALE: &str = r#"
 _   _ _       _     _   _                   _
| \ | (_) __ _| |__ | |_(_)_ __   __ _  __ _| | ___
|  \| | |/ _` | '_ \| __| | '_ \ / _` |/ _` | |/ _ \
| |\  | | (_| | | | | |_| | | | | (_| | (_| | |  __/
|_| \_|_|\__, |_| |_|\__|_|_| |_|\__, |\__,_|_|\___|
         |___/                   |___/
"#;

fn main() {
    println!("{NIGHTINGALE}\n");

    println!("Reading nightingale.yml");

    let file = match std::fs::File::open("nightingale.yml") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open nightingale.yml, {e}");
            return;
        }
    };

    let config = match serde_yaml::from_reader::<_, Config>(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read nightingale.yml, {e}");
            return;
        }
    };

    println!("Read nightingale.yml");

    if config.logging.enable {
        tracing_subscriber::fmt()
            .with_max_level(<LoggingLevel as Into<Level>>::into(config.logging.level))
            .init();
    }

    info!("Creating tokio runtime");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let _ = rt.block_on(entrypoint(config));

    info!("Shutting down Nightingale");

    rt.shutdown_timeout(Duration::from_secs(5));
}

async fn entrypoint(config: Config) -> Result<(), Error> {
    tokio::spawn(api::start_http(config));

    tokio::signal::ctrl_c().await?;
    info!("Ctrl C pressed");

    Ok(())
}
