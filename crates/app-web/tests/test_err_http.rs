use {
    app_base::prelude::*,
    app_web::ext::{ErrHttp, VoidHttp},
    core::error::Error
};

#[test]
fn test_err_http() {
    let err = (|| -> VoidHttp {
        (|| -> Void {
            (|| -> VoidHttp {
                (|| -> VoidAsync {
                    Err(std::io::Error::other("Foo"))?;
                    unreachable!();
                })()
                .map_err(Box::new)?;
                unreachable!();
            })()
            .map_err(Box::new)?;
            unreachable!();
        })()
        .map_err(Box::new)?;
        unreachable!();
    })()
    .unwrap_err();

    dbg!(&err);
    assert!(matches!(err, ErrHttp(..)));
    assert!(err.downcast_ref::<Box<Err>>().is_none());
    assert!(err.downcast_ref::<Box<ErrAsync>>().is_none());
    assert!(err.downcast_ref::<Box<ErrBox<dyn Error>>>().is_none());
    assert!(err.downcast_ref::<Box<ErrHttp>>().is_none());
    assert!(err.downcast_ref::<std::io::Error>().is_some());

    let err = Err::from(Box::new(err));
    dbg!(&err);
    assert!(matches!(err, ErrBox(..)));
    assert!(err.downcast_ref::<Box<ErrHttp>>().is_some());

    let err = ErrHttp::from(Box::new(err));
    dbg!(&err);
    assert!(matches!(err, ErrHttp(..)));
    assert!(err.downcast_ref::<std::io::Error>().is_some());
}
