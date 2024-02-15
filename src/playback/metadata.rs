use songbird::input::AuxMetadata;
use typemap_rev::TypeMapKey;
use crate::api::model::track::Track;


/// Data inserted to all track handles.
pub struct TrackMetadata {
    /// Metadata of the track.
    pub metadata: AuxMetadata,
    /// The guild the track belongs to.
    pub guild: u64
}

impl TrackMetadata {
    pub fn track(&self) -> Track {
        (&self.metadata).into()
    }
}

impl TypeMapKey for TrackMetadata {
    type Value = Self;
}
