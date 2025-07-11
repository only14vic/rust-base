use app_base::prelude::*;

#[derive(Debug, ExtendFromIter)]
pub struct AppOptions {
    pub clear_static_di: bool
}

impl Default for AppOptions {
    fn default() -> Self {
        Self { clear_static_di: true }
    }
}
