use {
    app_base::prelude::*,
    core::{
        fmt::Display,
        ops::{Deref, DerefMut}
    },
    serde::{Deserialize, Serialize},
    std::collections::HashMap
};

#[derive(Debug, Default, Clone, ExtendFromIter, Serialize, Deserialize)]
pub struct AuthModuleConfig {
    pub url: String,
    pub login: Option<String>,
    pub skip: Vec<String>,
    pub roles: Vec<String>
}

#[derive(Debug, Default)]
pub struct AuthModules {
    modules: IndexMap<String, AuthModuleConfig>
}

impl Display for AuthModules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        for (name, module) in self.iter() {
            buf.push_str(&format!(
                "{name}.url={url}
{name}.login={login}
{name}.skip={skip}
{name}.roles={roles}\n",
                url = &module.url,
                login = &module.login.as_ref().unwrap_or(&"".into()),
                skip = &module.skip.join(","),
                roles = &module.roles.join(",")
            ));
        }
        write!(f, "{}", buf.trim_end())
    }
}

impl Deref for AuthModules {
    type Target = IndexMap<String, AuthModuleConfig>;

    fn deref(&self) -> &Self::Target {
        &self.modules
    }
}

impl DerefMut for AuthModules {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.modules
    }
}

impl<'a> Extend<(&'a str, Option<&'a str>)> for AuthModules {
    fn extend<T: IntoIterator<Item = (&'a str, Option<&'a str>)>>(&mut self, iter: T) {
        let mut map = IndexMap::<&str, HashMap<&str, Option<&str>>>::default();

        for (str, value) in iter.into_iter() {
            if let Some((module, param)) = str.split_once(".") {
                if map.contains_key(module) == false {
                    map.insert(module, HashMap::default());
                }
                map.get_mut(module).unwrap().insert(param, value);
            }
        }

        for (name, params) in map {
            self.modules
                .insert(name.to_string(), AuthModuleConfig::from_iter(params));
        }

        self.modules
            .sort_by(|_, m1, _, m2| m2.url.len().cmp(&m1.url.len()));
    }
}

#[derive(Debug, Default, ExtendFromIter, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(skip)]
    pub modules: AuthModules
}

impl Iter<'_, (&'static str, String)> for AuthConfig {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [("web.auth.modules", &self.modules as &dyn Display)]
            .into_iter()
            .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for AuthConfig {
    fn init_args(&mut self, _args: &mut Args) {}

    fn load_args(&mut self, _args: &Args) {}
}

impl LoadEnv for AuthConfig {
    fn load_env(&mut self) {}
}
