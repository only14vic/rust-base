use {app::Config, app_base::prelude::*};

fn main() -> Void {
    dotenv(false);
    let mut log = Logger::init()?;
    let config = Config::load("app.ini")?;
    log.configure(&config.base.log)?;

    ok()
}
