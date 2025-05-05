mod common;

use {
    app_base::prelude::*,
    common::*,
    futures::{FutureExt, future::join_all, lock::Mutex, stream::FuturesUnordered},
    std::{sync::LazyLock, time::Duration},
    tokio::time::sleep
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_testing_test_async() -> Void {
    static RES: LazyLock<Mutex<Vec<u32>>> = LazyLock::new(Default::default);

    join_all(FuturesUnordered::from_iter([
        TEST.run(async {
            for i in 0..100 {
                RES.lock().await.push(i);
                sleep(Duration::from_micros(1)).await;
            }
            ok()
        })
        .boxed_local(),
        TEST.run(async {
            for i in 100..200 {
                RES.lock().await.push(i);
            }
            ok()
        })
        .boxed_local()
    ]))
    .await;

    assert!(RES.lock().await.is_sorted() == false);

    ok()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_testing_test_sync() -> Void {
    static RES: LazyLock<Mutex<Vec<u32>>> = LazyLock::new(Default::default);

    join_all(FuturesUnordered::from_iter([
        TEST.run_sync(0, async {
            for i in 0..100 {
                RES.lock().await.push(i);
                sleep(Duration::from_micros(1)).await;
            }
            ok()
        })
        .boxed_local(),
        TEST.run_sync(1, async {
            for i in 100..200 {
                RES.lock().await.push(i);
            }
            ok()
        })
        .boxed_local()
    ]))
    .await;

    assert!(RES.lock().await.is_sorted());

    ok()
}
