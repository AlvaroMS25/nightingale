use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use sysinfo::ProcessRefreshKind;
use crate::api::model::info::{CoreInfo, CpuInfo, Info, MemoryInfo, PlaybackInfo, SystemInfo};
use crate::api::state::State;

pub async fn info(AxumState(state): AxumState<State>) -> Result<impl IntoResponse, impl IntoResponse>{
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

    for s in state.instances.iter() {
        let session = s.value().read().await;
        players += session.playback.calls.len();

        for c in session.playback.calls.iter() {
            let call = c.read().await;

            if let Some(_) = call.queue().current() {
                playing += 1;
            }
        }
    }

    Ok(Json(Info {
        system: info,
        playback: PlaybackInfo {
            players: players as u64,
            playing
        },
    }))
}