use {app::Config, app_base::prelude::*, std::env::set_current_dir};

#[test]
fn tests_config() -> Void {
    set_current_dir(env!("PWD"))?;
    let _config = Config::load("app.ini")?;

    ok()
}
