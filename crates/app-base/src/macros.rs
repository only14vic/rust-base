pub use app_macros::*;

#[macro_export]
macro_rules! type_name_simple {
    ($ty:ty) => {
        type_name::<$ty>()
            .split("<")
            .next()
            .unwrap()
            .split("::")
            .last()
            .unwrap()
    };
}
