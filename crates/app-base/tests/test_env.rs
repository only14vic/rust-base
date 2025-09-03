use {
    app_base::prelude::*,
    core::hint::black_box,
    std::{sync::Barrier, time::Instant}
};

const THREADS_COUNT: usize = 4;
const MAX_ITERS: usize = 1_000_000;

#[test]
fn test_env() {
    let env = Env::default();
    dbg!(&env);

    let barrier = Barrier::new(THREADS_COUNT);
    let ta = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..THREADS_COUNT {
            s.spawn(|| {
                barrier.wait();
                let t = Instant::now();
                for _ in 0..MAX_ITERS {
                    black_box({
                        assert_eq!(env.env, std::env::var("APP_ENV").unwrap_or_default());
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
                        assert_eq!(env.env, Env::env());
                        assert_eq!(env.is_test, Env::is_test());
                        assert_eq!(env.is_dev, Env::is_dev());
                        assert_eq!(env.is_prod, Env::is_prod());
                        assert_eq!(env.is_debug, Env::is_debug());
                        assert_eq!(env.is_release, Env::is_release());
                    });
                }
                dbg!(t.elapsed());
            });
        }
    });
    println!("Env struct - time all: {:?}", ta.elapsed());
}
