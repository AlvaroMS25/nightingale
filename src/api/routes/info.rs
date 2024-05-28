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

        if player.queue.current().is_some() {
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

    // we know we will await the task we'll spawn next, so it is not necessary to increase the
    // arc refcount just to decrease it in a moment, so make a shared ptr out of the state,
    // and we won't deallocate the data later.
    let ptr = state.as_ptr();

    let handle = tokio::task::spawn_blocking(move || {
        ptr.system.update();
        ptr.system.get()
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