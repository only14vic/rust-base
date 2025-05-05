#[macro_export]
macro_rules! skip {
    (ok) => {
        skip!(() => Ok(()))
    };

    ($message:expr => $result:expr) => {
        log::warn!("Skipping {}", stringify!($message));
        return $result;
    };

    ($message:expr) => {
        skip!($message => "...")
    };

    ($message:expr,ok) => {
        skip!($message => Ok(()))
    };

    () => {
        skip!(() => "...")
    };
}
