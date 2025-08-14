use {
    app_base::prelude::*,
    serde::{Deserialize, Serialize},
    std::collections::HashMap
};

#[derive(Debug, Clone, ExtendFromIter, Serialize, Deserialize)]
pub struct HtmlRenderConfig {
    pub assets_dir: String,
    pub public_dir: String,
    pub pages_dir: String,
    pub index_file: String,
    pub files_glob: String,
    pub default_module: String,
    pub modules: HashMap<String, Option<String>>
}

impl Default for HtmlRenderConfig {
    fn default() -> Self {
        Self {
            assets_dir: "assets".into(),
            public_dir: "public".into(),
            pages_dir: "pages".into(),
            index_file: "index/index.html".into(),
            files_glob: "/**/*.html".into(),
            default_module: "/site/html".into(),
            modules: Default::default()
        }
    }
}

impl LoadDirs for HtmlRenderConfig {
    fn load_dirs(&mut self, dirs: &Dirs) -> Void {
        if dirs.data.is_empty() == false && self.assets_dir.starts_with("/") == false {
            self.assets_dir.insert(0, '/');
            self.assets_dir.insert_str(0, &dirs.data);
        }
        if dirs.data.is_empty() == false && self.public_dir.starts_with("/") == false {
            self.public_dir.insert(0, '/');
            self.public_dir.insert_str(0, &dirs.data);
        }
        ok()
    }
}

impl LoadEnv for HtmlRenderConfig {
    fn load_env(&mut self) -> Void {
        ok()
    }
}

impl LoadArgs for HtmlRenderConfig {
    fn load_args(&mut self, _args: &Args) -> Void {
        ok()
    }
}
