use {app_base::prelude::Env, core::hint::black_box, std::time::Instant};

#[test]
fn test_env() {
    let env = Env::default();
    dbg!(&env);

    let t = Instant::now();
    for _ in 0..1_000_000 {
        black_box({
            assert_eq!(env.is_test, Env::is_test());
            assert_eq!(env.is_dev, Env::is_dev());
            assert_eq!(env.is_prod, Env::is_prod());
            assert_eq!(env.is_debug, Env::is_debug());
            assert_eq!(env.is_release, Env::is_release());
        });
    }
    dbg!(t.elapsed());
}
