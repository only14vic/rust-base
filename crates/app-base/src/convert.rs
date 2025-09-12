use core::ops::Deref;

#[inline]
pub fn tuple_option_str<'a, T>(
    item: &'a (&'a str, Option<T>)
) -> (&'a str, Option<&'a str>)
where
    T: Deref<Target = str>
{
    (item.0, item.1.as_deref())
}

#[inline]
pub fn tuple_option_option_str<'a, T>(
    item: &'a (&'a str, Option<Option<T>>)
) -> (&'a str, Option<&'a str>)
where
    T: Deref<Target = str>
{
    (item.0, item.1.as_ref().unwrap_or(&None).as_deref())
}

#[inline]
pub fn tuple_result_option_str<'a, T, E>(
    item: &'a (&'a str, Result<Option<T>, E>)
) -> (&'a str, Option<&'a str>)
where
    T: Deref<Target = str>
{
    (item.0, item.1.as_ref().ok().unwrap_or(&None).as_deref())
}
