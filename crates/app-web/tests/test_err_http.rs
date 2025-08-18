use {
    actix_web::HttpResponse,
    app_base::prelude::*,
    app_web::ext::{ErrHttp, OkHttp}
};

#[test]
#[allow(unused_must_use)]
fn test_err_http() {
    let res = (|| -> OkHttp {
        (|| -> Void {
            (|| -> OkHttp {
                (|| -> Void {
                    Err("Foo")?;
                    ok()
                })()
                .map_err(Box::new)?;
                Ok(HttpResponse::Ok().finish())
            })()
            .map_err(Box::new)?;
            ok()
        })()
        .map_err(Box::new)?;
        Ok(HttpResponse::Ok().finish())
    })();

    assert!(matches!(res.unwrap_err(), ErrHttp(ErrBox(Box { .. }))));
}
