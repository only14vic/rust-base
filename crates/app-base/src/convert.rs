use alloc::string::String;

#[inline]
pub fn tuple_option_string_to_str<'a>(
    item: &'a (&'a str, Option<String>)
) -> (&'a str, Option<&'a str>) {
    (item.0, item.1.as_deref())
}

#[inline]
pub fn tuple_option_option_string_to_str<'a>(
    item: &'a (&'a str, Option<&Option<String>>)
) -> (&'a str, Option<&'a str>) {
    (item.0, item.1.unwrap_or(&None).as_deref())
}
