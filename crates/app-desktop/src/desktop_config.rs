use {
    app_base::prelude::*,
    core::fmt::Display,
    serde::{Deserialize, Serialize},
    std::sync::Arc
};

pub trait DesktopConfigExt: AppConfigExt + AsRef<Arc<DesktopConfig>> {}

#[derive(Debug, ExtendFromIter, Serialize, Deserialize)]
pub struct DesktopConfig {
    pub icon_path: String,
    pub webview_url: String,
    pub webview_start_url: String
}

impl AppConfigExt for DesktopConfig {
    const COMMAND: &str = "desktop";
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            icon_path: "assets/desktop/icon.png".into(),
            webview_url: option_env!("DESKTOP_WEBVIEW_URL")
                .unwrap_or("http://localhost")
                .into(),
            webview_start_url: option_env!("DESKTOP_WEBVIEW_START_URL")
                .or(option_env!("DESKTOP_WEBVIEW_URL"))
                .unwrap_or("http://localhost")
                .into()
        }
    }
}

impl Iter<'_, (&'static str, String)> for DesktopConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [
            ("desktop.icon_path", &self.icon_path as &dyn Display),
            ("desktop.webview_url", &self.webview_url),
            ("desktop.webview_start_url", &self.webview_start_url)
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for DesktopConfig {
    fn init_args(&mut self, _args: &mut Args) {
        //args.add_options([]).unwrap();
    }

    fn load_args(&mut self, _args: &Args) {
        //self.extend([].iter().map(convert::tuple_result_option_str));
    }
}

impl LoadEnv for DesktopConfig {
    fn load_env(&mut self) {
        //self.extend([].iter().map(convert::tuple_option_str));
    }
}

impl LoadDirs for DesktopConfig {
    fn load_dirs<'a>(&'a mut self, dirs: &'a Dirs) {
        if dirs.data.is_empty() == false && self.icon_path.starts_with("/") == false {
            self.icon_path.insert(0, '/');
            self.icon_path
                .insert_str(0, dirs.data.trim_end_matches('/'));
        }
    }
}
