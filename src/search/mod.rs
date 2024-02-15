pub mod youtube;

/// Helper that stores available search sources.
pub struct Search {
    pub youtube: youtube::YoutubeSearch
}

impl Search {
    pub fn new() -> Self {
        Self {
            youtube: youtube::YoutubeSearch::new()
        }
    }
}
