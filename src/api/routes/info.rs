use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use sysinfo::ProcessRefreshKind;
use crate::api::extractors::session::SessionExtractor;
use crate::api::model::info::{CoreInfo, CpuInfo, Info, MemoryInfo, PlaybackInfo, SystemInfo};
use crate::api::session::Session;
use crate::api::state::State;

async fn players_for(session: &Session) -> (u64, u64) {
    let mut playing = 0;

    for c in session.playback.players.iter() {
        let player = c.lock().await;

        if player.call.queue().current().is_some() {
            playing += 1;
        }
    }

    (session.playback.players.len() as u64, playing)
}

/// Retrieves information about the system running the server. If a session is provided,
/// information about the session is also sent, if not, information about all the sessions is sent.
pub async fn info(
    AxumState(state): AxumState<State>,
    session: Option<SessionExtractor>
) -> Result<impl IntoResponse, impl IntoResponse>{
    let state_clone = state.clone();
    let handle = tokio::task::spawn_blocking(move || {
        let mut lock = state_clone.system.lock().unwrap_or_else(|l| l.into_inner());

        lock.refresh_pids_specifics(&[state_clone.pid], ProcessRefreshKind::new()
            .with_cpu()
            .with_memory()
        );
        lock.refresh_cpu();
        lock.refresh_memory();

        let process = lock.process(state_clone.pid).unwrap();

        let cpu = CpuInfo {
            total_usage: lock.global_cpu_info().cpu_usage(),
            process_usage: process.cpu_usage(),
            cores: lock.cpus().into_iter()
                .map(|cpu| {
                    CoreInfo {
                        total_usage: cpu.cpu_usage(),
                        frequency: cpu.frequency()
                    }
                })
                .collect()
        };

        let mem = MemoryInfo {
            memory: process.memory(),
            virtual_memory: process.virtual_memory()
        };

        SystemInfo {
            cpu,
            memory: mem
        }

    }).await;

    let Ok(info) = handle else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let mut players = 0;
    let mut playing = 0;

    if let Some(SessionExtractor(s)) = session {
        (players, playing) = players_for(&s).await;
    } else {
        for s in state.instances.iter() {
            let p = players_for(&s).await;
            players += p.0;
            playing += p.1;
        }
    }

    Ok(Json(Info {
        system: info,
        playback: PlaybackInfo {
            players,
            playing
        },
    }))
}