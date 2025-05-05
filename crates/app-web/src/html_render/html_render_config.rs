use {
    app_base::prelude::*,
    core::fmt::Display,
    serde::{Deserialize, Serialize},
    std::collections::HashMap
};

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
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
            default_module: "site/html".into(),
            modules: Default::default()
        }
    }
}

impl Iter<'_, (&'static str, String)> for HtmlRenderConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [
            (
                "web.html_render.assets_dir", &self.assets_dir as &dyn Display
            ),
            ("web.html_render.public_dir", &self.public_dir),
            ("web.html_render.pages_dir", &self.pages_dir),
            ("web.html_render.index_file", &self.index_file),
            ("web.html_render.files_glob", &self.files_glob),
            ("web.html_render.default_module", &self.default_module),
            (
                "web.html_render.modules",
                Box::leak(Box::new(
                    self.modules
                        .iter()
                        .map(|(n, m)| format!("{n}={}", m.as_ref().unwrap_or(&"".into())))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            )
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for HtmlRenderConfig {
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("web-html-render-assets-dir", None, None),
            ("web-html-render-public-dir", None, None),
            ("web-html-render-pages-dir", None, None),
            ("web-html-render-index-file", None, None),
            ("web-html-render-files-glob", None, None),
            ("web-html-render-default-module", None, None)
        ])
        .unwrap();
    }

    fn load_args(&mut self, args: &Args) {
        self.extend(
            [
                ("assets_dir", args.get("web-html-render-assets-dir")),
                ("public_dir", args.get("web-html-render-public-dir")),
                ("pages_dir", args.get("web-html-render-pages-dir")),
                ("index_file", args.get("web-html-render-index-file")),
                ("files_glob", args.get("web-html-render-files-glob")),
                ("default_module", args.get("web-html-render-default-module"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
    }
}

impl LoadDirs for HtmlRenderConfig {
    fn load_dirs(&mut self, dirs: &Dirs) {
        if dirs.data.is_empty() == false && self.assets_dir.starts_with("/") == false {
            self.assets_dir.insert(0, '/');
            self.assets_dir.insert_str(0, &dirs.data);
        }
        if dirs.data.is_empty() == false && self.public_dir.starts_with("/") == false {
            self.public_dir.insert(0, '/');
            self.public_dir.insert_str(0, &dirs.data);
        }
    }
}

impl LoadEnv for HtmlRenderConfig {
    fn load_env(&mut self) {}
}
