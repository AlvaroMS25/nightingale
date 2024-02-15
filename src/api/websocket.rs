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
        let session_id = session.id;
        ws.on_upgrade(move |ws| initialize_websocket(state, ws, session_id, true))
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
            if let Some((_, s)) = state.instances.remove(&id) {
                s.destroy().await;
            }
        } else {
            *session.playback.receiver.lock() = Some(receiver);

            tokio::time::sleep(timeout).await;

            if session.playback.receiver.lock().is_some() {
                if let Some((_, s)) = state.instances.remove(&id) {
                    s.destroy().await;
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
    /// State of the server, currenly unused.
    state: State,
    /// Receiver used by the [`Sharder`](crate::playback::sharder::Sharder) to communicate
    /// with the clients via this handler.
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
                // this error is just a boxed tungstenite error.
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
            self.session.playback.process_event(msg.into()).await;
            debug!("Event forwarded");
            return;
        }

        // Other messages besides voice ones are not currently supported
        match msg {
            _ => {}
        }
    }

    async fn send_ready(&mut self, resume: bool) {
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
            .field("socket", &"WebSocket")
            .field("abort", &self.abort)
            .finish()
    }
}
