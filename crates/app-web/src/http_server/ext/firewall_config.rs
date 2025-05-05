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
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("web-firewall-fails-anon", None, None),
            ("web-firewall-fails-user", None, None),
            ("web-firewall-fails-period", None, None),
            ("web-firewall-total-fails", None, None),
            ("web-firewall-total-period", None, None)
        ])
        .unwrap();
    }

    fn load_args(&mut self, args: &Args) {
        self.extend(
            [
                ("fails_anon", args.get("web-firewall-fails-anon")),
                ("fails_user", args.get("web-firewall-fails-user")),
                ("fails_period", args.get("web-firewall-fails-period")),
                ("total_fails", args.get("web-firewall-total-fails")),
                ("total_period", args.get("web-firewall-total-period"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
    }
}

impl LoadEnv for FirewallConfig {
    fn load_env(&mut self) {}
}
