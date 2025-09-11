use {
    app_base::prelude::*,
    core::fmt::Display,
    serde::{Deserialize, Serialize}
};

#[derive(Default, Debug, Clone, ExtendFromIter, Serialize, Deserialize)]
pub struct FirewallConfig {
    pub fails_anon: u16,
    pub fails_user: u16,
    pub fails_period: u64,
    pub total_fails: u16,
    pub total_period: u64
}

impl Iter<'_, (&'static str, String)> for FirewallConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [
            ("web.firewall.fails_anon", &self.fails_anon as &dyn Display),
            ("web.firewall.fails_user", &self.fails_user),
            ("web.firewall.fails_period", &self.fails_period),
            ("web.firewall.total_fails", &self.total_fails),
            ("web.firewall.total_period", &self.total_period)
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for FirewallConfig {
    fn init_args(&mut self, _args: &mut Args) {}

    fn load_args(&mut self, _args: &Args) {}
}

impl LoadEnv for FirewallConfig {
    fn load_env(&mut self) {}
}
