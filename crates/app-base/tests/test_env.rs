use {
    app_base::prelude::*,
    core::hint::black_box,
    std::{sync::Barrier, time::Instant}
};

const THREADS_COUNT: usize = 4;
const MAX_ITERS: usize = 1_000_000;

#[test]
fn test_env() {
    dbg!(MAX_ITERS, Env::from_static());

    let barrier = Barrier::new(THREADS_COUNT);
    let ta = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..THREADS_COUNT {
            s.spawn(|| {
                barrier.wait();
                let t = Instant::now();
                for _ in 0..MAX_ITERS {
                    black_box({
                        assert_eq!(
                            Env::env(),
                            std::env::var("APP_ENV").unwrap_or_default()
                        );
                    });
                }
                dbg!(t.elapsed());
            });
        }
    });
    println!("std::env:var() - time all: {:?} \n", ta.elapsed());

    let barrier = Barrier::new(THREADS_COUNT);
    let ta = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..THREADS_COUNT {
            s.spawn(|| {
                barrier.wait();
                let t = Instant::now();
                for _ in 0..MAX_ITERS {
                    black_box({
                        assert_eq!(Env::env(), Env::env());
                        assert_eq!(Env::is_test(), Env::is_test());
                        assert_eq!(Env::is_dev(), Env::is_dev());
                        assert_eq!(Env::is_prod(), Env::is_prod());
                        assert_eq!(Env::is_debug(), Env::is_debug());
                        assert_eq!(Env::is_release(), Env::is_release());
                    });
                }
                dbg!(t.elapsed());
            });
        }
    });
    println!("Env struct - time all: {:?}", ta.elapsed());
}
