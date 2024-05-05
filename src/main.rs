use std::time::Duration;
use tracing::{error, info, Level};
use crate::config::{Config, LoggingLevel, LoggingOutput};
use crate::trace::TracingWriter;

mod playback;
mod api;
mod config;
mod channel;
mod abort;
mod metrics;
mod source;
mod ext;
mod ptr;
mod ticket;
mod mutex;
mod trace;

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

    println!("Reading nightingale.toml");

    let file = match std::fs::read_to_string("nightingale.toml") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to read nightingale.toml, {e}");
            return;
        }
    };

    let config = match toml::from_str::<Config>(&file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read nightingale.toml, {e}");
            return;
        }
    };

    println!("Read nightingale.toml");
    let mut _writer_guard = None;

    if config.logging.enable {
        let writer = match (config.logging.output, config.logging.file.as_ref()) {
            (LoggingOutput::StdOut, _) => Ok(TracingWriter::stdout()),
            (LoggingOutput::File, Some(path)) => TracingWriter::file(path.as_str()),
            (LoggingOutput::File, None) => {
                eprintln!("Logging output was set to `file`, but no `file` path was provided");
                return;
            }
        };

        let w = match writer {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create logging writer, error: {e}");
                return;
            }
        };

        let (nb, g) = tracing_appender::non_blocking(w);

        _writer_guard = Some(g);

        tracing_subscriber::fmt()
            .with_writer(nb)
            .with_max_level(<LoggingLevel as Into<Level>>::into(config.logging.level))
            .init();
    }

    info!("Creating tokio runtime");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    if which::which("yt-dlp").is_err() {
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
