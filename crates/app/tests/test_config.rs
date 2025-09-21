use {
    app::{App, Config, MODULE_CONFIG},
    app_base::prelude::*,
    common::TEST
};

mod common;

#[actix_web::test]
async fn test_config() -> Void {
    TEST.run(async {
        let mut app = App::new([MODULE_CONFIG]);

        app.with_args([("command", AppConfigModule::<Config>::COMMAND)])
            .boot()?
            .run()?;

        assert!(Env::is_test());
        assert!(app.config().db.url.contains("dbname=app-test"));

        ok()
    })
    .await
}
