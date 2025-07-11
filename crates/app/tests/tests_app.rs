use {app::App, app_base::prelude::*, std::env::set_current_dir};

#[test]
fn tests_app() -> Void {
    set_current_dir(env!("PWD"))?;
    let _app = App::boot()?;

    ok()
}
