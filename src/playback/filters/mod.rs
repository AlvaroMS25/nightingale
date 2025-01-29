pub struct FilterChain {
    filters: Vec<Box<dyn AudioFilter>>
}

impl FilterChain {
    pub fn new() -> Self {
        FilterChain {
            filters: Vec::new()
        }
    }

    pub fn add_filter(&mut self, filter: Box<dyn AudioFilter>) {
        self.filters.push(filter);
    }

    pub fn remove_filter(&mut self, filter: Box<dyn AudioFilter>) {
        self.filters.retain(|f| f.name() != filter.name());
    }
}

pub trait AudioFilter: Send + Sync + 'static {
    fn name(&self) -> &'static str;
}