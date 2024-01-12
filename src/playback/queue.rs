use std::collections::HashMap;
use songbird::tracks::TrackHandle;

#[derive(Default)]
pub struct Queue {
    inner: HashMap<u64, Vec<TrackHandle>>
}

impl Queue {
    pub fn for_guild(&mut self, guild: u64) -> &mut Vec<TrackHandle> {
        if self.inner.contains_key(&guild) {
            self.inner.get_mut(&guild).unwrap()
        } else {
            self.inner.insert(guild, Vec::new());
            self.inner.get_mut(&guild).unwrap()
        }
    }
}
