use std::time::Duration;
use tracing::{error, info, Level};
use crate::config::{Config, LoggingLevel};

mod playback;
mod api;
mod config;
mod channel;
mod abort;
mod metrics;
mod source;
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

    if let Err(_) = which::which("yt-dlp") {
        error!("yt-dlp executable couldn't be found, please install it for Nightingale to work properly");
        return;
    }

    let _ = rt.block_on(entrypoint(config));

    info!("Shutting down Nightingale");

    rt.shutdown_timeout(Duration::from_secs(5));
}

async fn entrypoint(config: Config) -> std::io::Result<()> {
    let mut handle = tokio::spawn(api::start_http(config));
    let ctrlc = tokio::signal::ctrl_c();

    tokio::pin!(ctrlc);

    tokio::select! {
        Ok(Err(res)) = &mut handle => {
            error!("Http server exited prematurely, error: {}", res.to_string());
        },
        _ = &mut ctrlc => {
            info!("Ctrl C pressed");
        }
    }

    Ok(())
}
