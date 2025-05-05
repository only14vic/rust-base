use {app::App, app_base::prelude::*, common::TEST};

mod common;

#[actix_web::test]
async fn tests_config() -> Void {
    TEST.run(async {
        let mut app = App::new([]);
        app.boot()?;

        assert!(app.config().db.url.contains("dbname=app-test"));

        ok()
    })
    .await
}
