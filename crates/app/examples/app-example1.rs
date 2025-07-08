use {app::Config, app_base::prelude::*};

fn main() -> Void {
    let config = Config::load()?;

    dbg!(&config);

    ok()
}
