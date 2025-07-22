use {app::App, app_base::prelude::*, common::TEST};

mod common;

#[actix_web::test]
async fn tests_app_config() -> Void {
    TEST.run(async {
        let app = App::boot()?;

        assert!(app.config().db.url.contains("dbname=app-test"));

        ok()
    })
    .await
}
