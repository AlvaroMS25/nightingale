use std::time::Duration;
use base64::Engine;
use tracing::{error, info, Level};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::filter::LevelFilter;
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
mod system;

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
    let mut loki_task = None;

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

        if config.metrics.enable_loki {
            if config.loki.is_none() {
                eprintln!("Loki is enabled, but no `loki` config was provided");
                return;
            }

            let opts = config.loki.as_ref().unwrap();

            let loki_auth = base64::engine::general_purpose::STANDARD
                .encode(format!("{}:{}", opts.user, opts.password));

            let (layer, task) = tracing_loki::builder()
                .label("application", "Nightingale")
                .unwrap()
                .http_header("Authorization", format!("Basic {loki_auth}"))
                .unwrap()
                .build_url(opts.url.parse().expect("invalid Loki url provided"))
                .unwrap();


            loki_task = Some(task);

            let filter: LevelFilter = <LoggingLevel as Into<Level>>::into(config.logging.level).into();

            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer()
                    .with_writer(nb)
                )
                .with(layer)
                .init();
        } else {
            tracing_subscriber::fmt()
                .with_writer(nb)
                .with_max_level(<LoggingLevel as Into<Level>>::into(config.logging.level))
                .init();
        }
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

    if let Some(t) = loki_task {
        info!("Starting Loki subscriber");
        rt.spawn(t);
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
