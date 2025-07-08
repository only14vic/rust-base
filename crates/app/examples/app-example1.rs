use {app::Config, app_base::prelude::*};

fn main() -> Void {
    let config = Config::load("app.ini")?;

    dbg!(&config);

    ok()
}
