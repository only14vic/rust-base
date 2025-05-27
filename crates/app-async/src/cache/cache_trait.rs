use {
    app_base::prelude::*,
    std::{future::Future, sync::Arc}
};

pub trait Cache {
    fn get_key(&self, group: &str, keys: &[&str]) -> String {
        let mut str = String::with_capacity(
            group.len() + keys.len() + keys.iter().fold(0, |acc, &key| acc + key.len())
        );

        if str.capacity() == 0 {
            return str;
        }

        str.push_str(group);
        str.push(':');

        keys.iter().for_each(|key| {
            str.push_str(key);
            str.push(':');
        });

        str
    }

    async fn get<T: Send + Sync + 'static>(
        &self,
        group: &str,
        keys: &[&str]
    ) -> Ok<Option<Arc<T>>>;

    async fn set<T: Send + Sync + 'static>(
        &self,
        group: &str,
        keys: &[&str],
        value: T,
        lifetime: u64
    ) -> Ok<Option<Arc<T>>>;

    async fn exists(&self, group: &str, keys: &[&str]) -> Ok<bool>;

    async fn len(&self) -> usize;

    async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    async fn keys(&self) -> Vec<String>;

    async fn remove(&self, group: &str, keys: &[&str]) -> Void;

    async fn remove_expired(&self) -> Ok<usize>;

    async fn clear(&self, group: &str, keys: &[&str]) -> Void;

    async fn clear_all(&self) -> Void;

    async fn call<T: Send + Sync + 'static>(
        &self,
        group: &str,
        keys: &[&str],
        lifetime: u64,
        callback: impl Future<Output = Ok<T>>
    ) -> Ok<Option<Arc<T>>>;
}
