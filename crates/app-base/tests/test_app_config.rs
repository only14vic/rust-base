use {
    app_base::{app::*, prelude::*},
    std::env::set_current_dir
};

type MyConfig = AppConfig<AppSimpleConfig>;

#[test]
fn test_app_config() -> Void {
    set_current_dir(env!("PWD"))?;

    let mut config = MyConfig::default();

    dotenv(false);
    log_init();

    let mut args = Args::new([
        ("exe", "0".into(), None),
        ("command", "1".into(), Some(MyConfig::COMMAND)),
        ("help", "-h".into(), None),
        ("base-locales", None, Some(" fr = fr_US ")),
        ("base-language", None, Some("fr")),
        ("custom", None, Some("Bar"))
    ])
    .unwrap();
    args.set_undefined(ArgUndef::Add);
    args.parse_args(std::env::args().collect())?;

    config.load(Some(&args))?;

    let config_dump: Vec<(&str, String)> = config.iter().collect();

    dbg!(&args, &config);

    assert!(config_dump.contains(&("custom", "Bar".into())));

    assert_eq!(
        Some((&"fr".to_owned(), &Some("fr_US".to_owned()))),
        config.base.locales.get_key_value("fr")
    );
    assert_eq!("fr", &config.base.language);

    assert_eq!(
        Some("Bar".into()),
        config.custom.as_ref().map(|v| v.as_ref())
    );

    ok()
}
