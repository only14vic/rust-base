use {alloc::string::String, core::ops::Deref};

#[inline]
pub fn tuple_option_string_to_str<'a>(
    item: &'a (&'a str, Option<String>)
) -> (&'a str, Option<&'a str>) {
    (item.0, item.1.as_deref())
}

#[inline]
pub fn tuple_option_option_string_to_str<'a, T>(
    item: &'a (&'a str, Option<&Option<T>>)
) -> (&'a str, Option<&'a str>)
where
    T: Deref<Target = str>
{
    (item.0, item.1.unwrap_or(&None).as_deref())
}
