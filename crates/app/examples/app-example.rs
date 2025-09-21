use app_base::prelude::*;

type Config = AppSimpleConfig;

const CONFIG_MODULE: AppModule<Config> = AppConfigModule::<Config>::handle;
const SIMPLE_MODULE: AppModule<Config> = AppSimpleModule::<Config>::handle;

fn main() -> Void {
    App::<Config>::new([SIMPLE_MODULE, CONFIG_MODULE])
        .boot()?
        .run()
}
