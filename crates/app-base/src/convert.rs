use core::ops::Deref;

#[inline]
pub fn tuple_option_str<'a, T>(item: &'a (&'a str, Option<T>)) -> (&'a str, Option<&'a str>)
where
    T: Deref<Target = str>
{
    (item.0, item.1.as_deref())
}

#[inline]
pub fn tuple_option_option_str<'a, T>(
    item: &'a (&'a str, Option<&Option<T>>)
) -> (&'a str, Option<&'a str>)
where
    T: Deref<Target = str>
{
    (item.0, item.1.unwrap_or(&None).as_deref())
}
