use {
    crate::filters::filter_phone,
    alloc::string::String,
    serde::{Deserialize, Deserializer}
};

pub fn phone<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>
{
    let s: String = Deserialize::deserialize(d)?;
    Ok(filter_phone(&s))
}

pub fn phone_option<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>
{
    let s: Option<String> = Deserialize::deserialize(d)?;
    match s.as_ref() {
        Some(value) => {
            match filter_phone(value) {
                value if !value.is_empty() => Ok(Some(value)),
                _ => Ok(None)
            }
        },
        None => Ok(None)
    }
}
