use {
    app_base::prelude::*,
    futures::{FutureExt, future::LocalBoxFuture, lock::Mutex},
    futures_lite::future::yield_now,
    std::{
        future::Future,
        panic::AssertUnwindSafe,
        sync::atomic::{AtomicU32, Ordering}
    }
};

pub type TestFuture<'a, T> = LocalBoxFuture<'a, T>;
pub type FnTestFuture<'a, T> = fn() -> TestFuture<'a, T>;

pub struct Test<'a> {
    init: Mutex<Option<FnTestFuture<'a, ()>>>,
    before: Option<FnTestFuture<'a, ()>>,
    after: Option<FnTestFuture<'a, ()>>,
    order: AtomicU32
}

impl<'a> Test<'a> {
    pub fn new() -> Self {
        Self {
            init: Mutex::new(None),
            before: None,
            after: None,
            order: AtomicU32::new(0)
        }
    }

    pub fn init(mut self, fut: FnTestFuture<'a, ()>) -> Self {
        self.init = Mutex::new(Some(fut));
        self
    }

    pub fn before(mut self, fut: FnTestFuture<'a, ()>) -> Self {
        self.before = Some(fut);
        self
    }

    pub fn after(mut self, fut: FnTestFuture<'a, ()>) -> Self {
        self.after = Some(fut);
        self
    }

    pub async fn run<R>(&self, test: impl Future<Output = Ok<R>> + 'a) -> Ok<R> {
        self.run_inner(None, Box::pin(test)).await
    }

    pub async fn run_sync<R>(
        &self,
        order: u32,
        test: impl Future<Output = Ok<R>> + 'a
    ) -> Ok<R> {
        self.run_inner(Some(order), Box::pin(test)).await
    }

    async fn run_inner<R>(
        &self,
        order: Option<u32>,
        test: TestFuture<'a, Ok<R>>
    ) -> Ok<R> {
        let mut lock = if let Some(order) = order {
            loop {
                if let Some(lock) = self.init.try_lock()
                    && order == self.order.load(Ordering::Acquire)
                {
                    break lock;
                }

                yield_now().await;
            }
        } else {
            self.init.lock().await
        };

        if let Some(init) = lock.take() {
            init().await;
        }

        if let Some(before) = self.before.as_ref() {
            before().await;
        }

        let res = AssertUnwindSafe(test).catch_unwind().await;

        if let Some(after) = self.after.as_ref() {
            after().await;
        }

        if order.is_some() {
            self.order.fetch_add(1, Ordering::SeqCst);
        }

        yield_now().await;

        match res {
            Ok(res) => res,
            Err(..) => Err("Test panic catched")?
        }
    }
}
