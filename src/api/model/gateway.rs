use crate::api::model::track::Track;
use crate::api::model;

/// Events sent via websocket to clients.
#[non_exhaustive]
#[derive(serde::Serialize, Debug)]
#[serde(tag = "op", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Outgoing {
    /// Event sent on client websocket connection.
    Ready(model::ready::Ready),
    /// Track related events.
    Event {
        /// Guild id corresponding to the event.
        guild_id: u64,
        /// The event itself.
        event: OutgoingEvent
    },
    /// Gateway connection related events.
    UpdateState(super::connection::UpdateState)
}

/// Track related events.
#[non_exhaustive]
#[derive(serde::Serialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum OutgoingEvent {
    /// A track started its playback.
    TrackStart(Track),
    /// A track ended, either naturally or manually
    TrackEnd {
        /// Whether the track was stopped manually.
        stopped: bool,
        /// The track itself.
        track: Track
    },
    /// A track had an error on playback.
    TrackErrored {
        /// The error message.
        error: String,
        /// The track itself.
        track: Track
    }
}

