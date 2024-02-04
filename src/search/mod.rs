pub mod youtube;

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
