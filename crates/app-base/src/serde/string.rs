use {
    crate::alloc::string::ToString,
    alloc::string::String,
    core::{fmt::Display, str::FromStr},
    serde::{
        Deserialize, Deserializer, Serialize, Serializer,
        de::{Error, IntoDeserializer}
    },
    trim_in_place::TrimInPlace
};

pub fn skip_empty_string<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default
{
    let opt = Option::<String>::deserialize(d).unwrap_or_default();

    match opt.as_deref() {
        None => Ok(None),
        Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some)
    }
}

pub fn skip_empty_string_trim<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default
{
    let mut opt = Option::<String>::deserialize(d).unwrap_or_default();

    match opt.as_mut().map(String::trim_in_place) {
        None => Ok(None),
        Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some)
    }
}

pub fn default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default
{
    Ok(T::deserialize(d).unwrap_or_default())
}

pub fn parse_option<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + FromStr,
    <T as FromStr>::Err: Display
{
    let mut s = serde_json::Value::deserialize(d)?.to_string();
    s.trim_matches_in_place('"');

    if s.is_empty() {
        Ok(None)
    } else {
        match s.parse::<T>() {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(D::Error::custom(e))
        }
    }
}

pub fn parse<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + FromStr,
    <T as FromStr>::Err: Display
{
    let mut s = serde_json::Value::deserialize(d)?.to_string();
    s.trim_matches_in_place('"');

    s.parse::<T>().map_err(D::Error::custom)
}

pub fn to_string<S, T>(v: &T, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize
{
    s.serialize_str(
        &serde_json::to_string(&v).expect("Unable serialize value to string.")
    )
}
