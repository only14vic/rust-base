use {
    super::Cache,
    std::{ops::Deref, sync::Arc}
};

#[derive(Debug, Clone)]
pub struct Cacher<C: Cache> {
    cache: Arc<C>
}

impl<C: Cache> Cacher<C> {
    pub fn with(cache: C) -> Self {
        Self { cache: cache.into() }
    }
}

impl<C: Cache> Deref for Cacher<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.cache.as_ref()
    }
}
