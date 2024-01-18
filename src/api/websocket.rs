use std::fmt;
use std::future::Future;
use std::num::NonZeroU64;
use std::sync::Arc;
use axum::body::Body;
use axum::Error;
use axum::extract::{Query, State as AxumState, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::StreamExt;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tracing::{debug, info, trace, warn};
use uuid::Uuid;
use crate::abort::Abort;
use crate::api::model::gateway::{Incoming, Outgoing};
use crate::api::session::Session;
use crate::api::state::State;
use crate::tri;
use crate::api::extractors::session::{SESSION_NOT_PRESENT, SessionExtractor};
use crate::api::model::ready::Ready;
use crate::channel::Receiver;

#[derive(serde::Deserialize)]
pub struct SessionQuery {
    pub session: Uuid
}

#[derive(serde::Deserialize)]
pub struct ConnectQuery {
    pub shards: u64,
    pub user_id: NonZeroU64
}

pub async fn connect(
    AxumState(state): AxumState<State>,
    ws: WebSocketUpgrade,
    Query(options): Query<ConnectQuery>
) -> impl IntoResponse {
    let id = state.generate_uuid();

    state.instances.insert(id, Arc::new(RwLock::new(Session::new(id, options.shards, options.user_id))));

    ws.on_upgrade(move |ws| initialize_websocket(state, ws, id, false))
}

pub async fn resume(
    AxumState(state): AxumState<State>,
    ws: WebSocketUpgrade,
    SessionExtractor(session): SessionExtractor
) -> impl IntoResponse {
    if session.read().await.playback.receiver.is_none() {
      Response::builder()
          .status(StatusCode::CONFLICT)
          .body(Body::from(r#"{"message": "session taken"}"#))
          .unwrap()
    } else {
        let session_id = session.read().await.id;
        ws.on_upgrade(move |ws| initialize_websocket(state, ws, session_id, true))
    }
}

pub async fn initialize_websocket(state: State, websocket: WebSocket, id: Uuid, resume: bool) {
    let session = state.instances.get(&id).map(|s| Arc::clone(s.value())).unwrap();

    tokio::spawn(async move {
        let mut receiver = session.write().await.playback.receiver.take().unwrap();

        WebSocketHandler {
            id,
            socket: websocket,
            state: state.clone(),
            receiver: &mut receiver,
            session: Arc::clone(&session),
            abort: Abort::new()
        }.run(resume).await;

        info!("Websocket connection finished");

        let mut lock = session.write().await;
        lock.playback.receiver = Some(receiver);

        if !lock.options.enable_resume {
            drop(lock);
            state.instances.remove(&id);
        } else {
            let timeout = lock.options.timeout;
            drop(lock);

            tokio::time::sleep(timeout).await;

            let lock = session.read().await;
            if lock.playback.receiver.is_some() {
                state.instances.remove(&id);
            }
        }
    });
}

struct WebSocketHandler<'a> {
    id: Uuid,
    socket: WebSocket,
    state: State,
    receiver: &'a mut Receiver,
    session: Arc<RwLock<Session>>,
    abort: Abort
}

impl WebSocketHandler<'_> {
    #[tracing::instrument(skip(resume))]
    async fn run(mut self, resume: bool) {
        info!("Websocket connection established");
        self.resume_if_needed(resume).await;
        let mut abort = self.abort.as_future();
        loop {
            tokio::select! {
                biased;
                _ = &mut abort => {
                    todo!("cleanup")
                },
                Some(msg) = self.receiver.next() => {
                    self.send(msg).await;
                },
                Some(msg) = self.socket.next() => {
                    self.handle_possible_error(msg).await;
                }
            }
        }
    }

    async fn handle_possible_error(&mut self, msg: Result<Message, Error>) {
        match msg {
            Ok(msg) => match msg {
                Message::Text(msg) => match serde_json::from_str::<Incoming>(&msg) {
                    Err(error) => tracing::error!("Invalid payload received, error: {error}"),
                    Ok(incoming) => self.handle_message(incoming).await
                },
                Message::Close(frame) => {
                    info!("Close message received, frame: {frame:?}");
                    self.abort.abort()
                },
                _ => {}
            },
            Err(error) => {
                let error = error.into_inner().downcast::<tungstenite::Error>().unwrap();

                warn!("Error ocurred during connection: {error}");
                self.abort.abort();
            }
        }
    }

    async fn handle_message(&mut self, msg: Incoming) {
        debug!("Received message: {msg:?}");
        if msg.is_voice_event() {
            debug!("Received a voice event, forwarding to songbird");
            self.session.read().await.playback.songbird.process(&msg.into()).await;
            return;
        }

        match msg {
            _ => {}
        }
    }

    async fn resume_if_needed(&mut self, resume: bool) {
        self.send(Outgoing::Ready(Ready {
            resumed: resume,
            session: self.id
        })).await
    }

    async fn send(&mut self, value: Outgoing) {
        tri!(self.socket.send(Message::Text(tri!(serde_json::to_string(&value)))).await)
    }
}

impl fmt::Debug for WebSocketHandler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebSocketHandler")
            .field("id", &self.id)
            .field("socket", &self.socket)
            .field("abort", &self.abort)
            .finish()
    }
}
