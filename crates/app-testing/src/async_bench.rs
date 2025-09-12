use {
    criterion::async_executor::AsyncExecutor,
    std::{cell::RefCell, future::Future}
};

/// Actix runtime for criterion benchmars
/// ```no_run
/// use {
///     actix_web::{
///         App, HttpResponse,
///         http::Method,
///         rt::time::Instant,
///         test::{TestRequest, call_service, init_service},
///         web
///     },
///     app_testing::async_bench::ActixRuntime,
///     criterion::{Criterion, black_box, criterion_group, criterion_main}
/// };
///
/// fn action(c: &mut Criterion) {
///     c.bench_function("test-name", |b| {
///         b.to_async(ActixRuntime::new()).iter_custom(|iters| {
///             async move {
///                 let app = init_service(App::new().configure(|config| {
///                     config.service(web::resource("/test").route(web::to(|| {
///                         async { HttpResponse::Ok().body("Test") }
///                     })));
///                 }))
///                 .await;
///
///                 let start = Instant::now();
///                 for _i in 0..iters {
///                     black_box({
///                         let req = TestRequest::default().uri("/test").to_request();
///
///                         let res = call_service(&app, req).await;
///                         assert!(res.status().is_success());
///                     });
///                 }
///                 start.elapsed()
///             }
///         });
///     });
/// }
///
/// criterion_group!(benches, action);
/// criterion_main!(benches);
/// ```
pub struct ActixRuntime {
    rt: RefCell<actix_web::rt::Runtime>
}

impl ActixRuntime {
    pub fn new() -> Self {
        ActixRuntime { rt: RefCell::new(actix_web::rt::Runtime::new().unwrap()) }
    }
}

impl AsyncExecutor for ActixRuntime {
    fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        self.rt.borrow_mut().block_on(future)
    }
}
