use std::time::Duration;
use songbird::input::AuxMetadata;
use typemap_rev::TypeMapKey;
use crate::api::model::track::Track;


pub struct TrackMetadata {
    pub metadata: AuxMetadata,
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
