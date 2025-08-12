use app_base::prelude::*;

#[derive(Default, Debug, ExtendFromIter)]
pub struct FirewallConfig {
    pub fails_anon: u16,
    pub fails_user: u16,
    pub fails_period: u64,
    pub total_fails: u16,
    pub total_period: u64
}

impl LoadEnv for FirewallConfig {
    fn load_env(&mut self) -> Void {
        ok()
    }
}

impl LoadArgs for FirewallConfig {
    fn load_args(&mut self, _args: &Args) -> Void {
        ok()
    }
}
