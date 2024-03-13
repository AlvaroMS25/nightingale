use std::fmt;
use std::num::NonZeroU64;
use std::sync::Arc;
use axum::body::Body;
use axum::Error;
use axum::extract::{Query, State as AxumState, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use uuid::Uuid;
use crate::abort::Abort;
use crate::api::model::gateway::{Incoming, Outgoing};
use crate::api::session::Session;
use crate::api::state::State;
use crate::tri;
use crate::api::extractors::session::SessionExtractor;
use crate::api::model::ready::Ready;
use crate::channel::Receiver;

/// Query used on [`connect`].
#[derive(serde::Deserialize)]
pub struct ConnectQuery {
    /// Number of shards the client has, needed to properly
    /// forward messages through discord's gateway.
    pub shards: u64,
    /// The user id of the client.
    pub user_id: NonZeroU64
}

/// Opens a websocket connection and creates a new session.
pub async fn connect(
    AxumState(state): AxumState<State>,
    ws: WebSocketUpgrade,
    Query(options): Query<ConnectQuery>
) -> impl IntoResponse {
    let id = state.generate_uuid();

    // Create new session.
    state.instances.insert(id, Arc::new(Session::new(id, options.shards, options.user_id)));

    ws.on_upgrade(move |ws| initialize_websocket(state, ws, id, false))
}

/// Tries to resume an existing session, if the session already has a client connected, returns
/// a 409 Conflict.
pub async fn resume(
    AxumState(state): AxumState<State>,
    ws: WebSocketUpgrade,
    SessionExtractor(session): SessionExtractor
) -> impl IntoResponse {
    // Only one connection per session is allowed at a time, so if
    // the receiver is missing, the connection is already ongoing.
    if session.playback.receiver.lock().is_none() {
      Response::builder()
          .status(StatusCode::CONFLICT)
          .body(Body::from(r#"{"message": "session taken"}"#))
          .unwrap()
    } else {
        if let Some(token) = session.cleanup.lock().take() {
            token.cancel(); // Tell the cancellation task to exit
        }
        ws.on_upgrade(move |ws| initialize_websocket(state, ws, session.id, true))
    }
}

/// Initializes and cleans a websocket connection.
pub async fn initialize_websocket(state: State, websocket: WebSocket, id: Uuid, resume: bool) {
    let session = state.instances.get(&id).map(|s| Arc::clone(s.value())).unwrap();

    tokio::spawn(async move {
        let mut receiver = session.playback.receiver.lock().take().unwrap();

        WebSocketHandler {
            id,
            socket: websocket,
            state: state.clone(),
            receiver: &mut receiver,
            session: Arc::clone(&session),
            abort: Abort::new()
        }.run(resume).await;

        info!("Websocket connection finished");

        let (enable_resume, timeout) = {
            let lock = session.options.lock();
            (lock.enable_resume, lock.timeout)
        };

        if !enable_resume {
            debug!("Session[{id}] not allowed to resume, cleaning up");
            if let Some((_, s)) = state.instances.remove(&id) {
                s.destroy().await;
            }
        } else {
            debug!("Session[{id}] is allowed to resume, waiting {timeout:?} before cleaning up");
            *session.playback.receiver.lock() = Some(receiver);
            let token = CancellationToken::new();
            let future = token.clone().cancelled_owned();
            *session.cleanup.lock() = Some(token);

            match tokio::time::timeout(timeout, future).await {
                Ok(_) => {
                    debug!("Session[{id}] was resumed");
                },
                Err(_) => {
                    debug!("Session[{id}] was not resumed, cleaning up");
                    if let Some((_, s)) = state.instances.remove(&id) {
                        s.destroy().await;
                    }
                }
            }
        }
    });
}
/// Handler of a websocket connection, handlers and sessions have a 1:1 relationship,
/// so a handler manages a single session(and a session is managed by a single handler) at a time.
///
/// If a client wants to manage multiple sessions at once, a connection per session must be established
struct WebSocketHandler<'a> {
    /// Session id.
    id: Uuid,
    /// The socket itself.
    socket: WebSocket,
    #[allow(unused)]
    /// State of the server, currently unused.
    state: State,
    /// Receiver used by the sharder and event handlers to forward payloads
    /// to this handler clients.
    receiver: &'a mut Receiver,
    /// The session managed by the handler.
    session: Arc<Session>,
    /// Abort used to manually stop the handler.
    abort: Abort
}

impl WebSocketHandler<'_> {
    #[tracing::instrument(skip(resume))]
    async fn run(mut self, resume: bool) {
        info!("Websocket connection established");
        self.send_ready(resume).await;
        let mut abort = self.abort.as_future();
        loop {
            tokio::select! {
                biased;
                _ = &mut abort => {
                    let _ = self.socket.close().await;
                    return;
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
                Message::Close(frame) => {
                    info!("Close message received, frame: {frame:?}");
                    self.abort.abort()
                },
                _ => {}
            },
            Err(error) => {
                // this error is just a boxed tungstenite error.
                let error = error.into_inner().downcast::<tungstenite::Error>().unwrap();

                warn!("Error occurred during connection: {error}");
                self.abort.abort();
            }
        }
    }

    async fn send_ready(&mut self, resume: bool) {
        let players = if resume {
            let mut players = Vec::with_capacity(self.session.playback.players.len());

            for player in self.session.playback.players.iter() {
                players.push(player.lock().await.as_json().await)
            }

            Some(players)
        } else {
            None
        };

        self.send(Outgoing::Ready(Ready {
            resumed: resume,
            session: self.id,
            players
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
            .field("socket", &"WebSocket")
            .field("abort", &self.abort)
            .finish()
    }
}
