use {
    app_base::prelude::*,
    std::{future::Future, sync::Arc}
};

pub trait Cache {
    fn get_key(&self, keys: &[&str]) -> String {
        let mut str =
            String::with_capacity(keys.iter().fold(0, |acc, &key| acc + key.len() + 1));

        keys.iter().for_each(|key| {
            str.push_str(key);
            str.push(':');
        });

        str
    }

    async fn get<T: Send + Sync + 'static>(
        &self,
        keys: &[&str]
    ) -> OkAsync<Option<Arc<T>>>;

    async fn set<T: Send + Sync + 'static>(
        &self,
        keys: &[&str],
        value: T,
        lifetime: u64
    ) -> OkAsync<Option<Arc<T>>>;

    async fn exists(&self, keys: &[&str]) -> OkAsync<bool>;

    async fn len(&self) -> usize;

    async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    async fn keys(&self) -> Vec<String>;

    async fn remove(&self, keys: &[&str]) -> VoidAsync;

    async fn remove_expired(&self) -> OkAsync<usize>;

    async fn clear(&self, keys: &[&str]) -> VoidAsync;

    async fn clear_all(&self) -> VoidAsync;

    async fn getset<T: Send + Sync + 'static>(
        &self,
        keys: &[&str],
        lifetime: u64,
        callback: impl Future<Output = OkAsync<T>>
    ) -> OkAsync<Option<Arc<T>>>;
}
