use app_base::prelude::*;

#[derive(Debug, Default, Clone, ExtendFromIter)]
pub struct AuthModuleConfig {
    pub url: String,
    pub login: Option<String>,
    pub skip: Vec<String>,
    pub roles: Vec<String>
}

#[derive(Debug, Default, ExtendFromIter)]
pub struct AuthModules {
    pub profile: AuthModuleConfig,
    pub admin: AuthModuleConfig,
    pub admin_users: AuthModuleConfig
}

impl AuthModules {
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, AuthModuleConfig)> + use<> {
        [
            ("profile", self.profile.clone()),
            ("admin", self.admin.clone()),
            ("admin_users", self.admin_users.clone())
        ]
        .into_iter()
    }
}

#[derive(Debug, Default, ExtendFromIter)]
pub struct AuthConfig {
    pub modules: AuthModules
}

impl LoadEnv for AuthConfig {
    fn load_env(&mut self) -> Void {
        ok()
    }
}

impl LoadArgs for AuthConfig {
    fn load_args(&mut self, _args: &Args) -> Void {
        ok()
    }
}
