use {
    app_base::prelude::*,
    serde::{Deserialize, Serialize}
};

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct AppOptions {
    pub clear_static_di: bool
}

impl Default for AppOptions {
    fn default() -> Self {
        Self { clear_static_di: true }
    }
}
