#![allow(unused)]

use {
    ahash::AHasher,
    alloc::{
        boxed::Box,
        string::{String, ToString}
    },
    core::{
        any::type_name_of_val,
        error::Error,
        fmt::{self, Debug, Display},
        hash::BuildHasherDefault,
        ops::Deref
    },
    serde::{de::DeserializeOwned, Deserialize, Serialize}
};

pub type IndexMap<K, V, S = BuildHasherDefault<AHasher>> = indexmap::IndexMap<K, V, S>;
pub type IndexSet<V, S = BuildHasherDefault<AHasher>> = indexmap::IndexSet<V, S>;

#[derive(PartialEq, Eq)]
pub struct ErrBox<E: ?Sized>(Box<E>);

pub type Err = ErrBox<dyn Error>;
pub type ErrAsync = ErrBox<dyn Error + Send + Sync>;

impl<E: ?Sized> ErrBox<E> {
    pub fn take(self) -> Box<E> {
        self.0
    }
}

impl Err {
    #[inline(always)]
    pub fn new(error: Box<dyn Error>) -> Self {
        Self(error)
    }
}
impl ErrAsync {
    #[inline(always)]
    pub fn new(error: Box<dyn Error + Send + Sync>) -> Self {
        Self(error)
    }
}

impl Deref for Err {
    type Target = dyn Error;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl Deref for ErrAsync {
    type Target = dyn Error + Send + Sync;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Error for Box<Err> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}
impl Error for Box<ErrAsync> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

impl Debug for Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ErrBox<{}>: \"{}\"",
            type_name_of_val(self.0.as_ref()),
            &self.0
        )
    }
}
impl Debug for ErrAsync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ErrBox<{}>: \"{}\"",
            type_name_of_val(self.0.as_ref()),
            &self.0
        )
    }
}

impl Display for Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_ref())
    }
}
impl Display for ErrAsync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_ref())
    }
}

impl Error for &'static Err {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}
impl Error for &'static ErrAsync {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

impl AsRef<Box<dyn Error>> for Err {
    fn as_ref(&self) -> &Box<dyn Error> {
        &self.0
    }
}
impl AsRef<Box<dyn Error + Send + Sync>> for ErrAsync {
    fn as_ref(&self) -> &Box<dyn Error + Send + Sync> {
        &self.0
    }
}

impl<T: Into<Box<dyn Error>>> From<T> for Err {
    #[inline(always)]
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}
impl<T: Into<Box<dyn Error + Send + Sync>>> From<T> for ErrAsync {
    #[inline(always)]
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}

impl From<ErrAsync> for Err {
    #[inline(always)]
    fn from(value: ErrAsync) -> Self {
        Self(value.0)
    }
}
impl From<Err> for ErrAsync {
    #[inline(always)]
    fn from(value: Err) -> Self {
        Self::new(value.0.to_string().into())
    }
}

pub type Ok<T> = Result<T, Err>;
pub type OkAsync<T> = Result<T, ErrAsync>;
pub type Void = Ok<()>;
pub type VoidAsync = OkAsync<()>;

#[inline(always)]
pub const fn ok<E>() -> Result<(), E> {
    Ok(())
}

impl<T: Sized> BaseFromInto for T {}

pub trait BaseFromInto
where
    Self: Sized
{
    #[inline(always)]
    fn into_ok<T: From<Self>, E>(self) -> Result<T, E> {
        Ok(self.into())
    }

    #[inline(always)]
    fn into_some<T: From<Self>>(self) -> Option<T> {
        Some(self.into())
    }

    #[inline(always)]
    fn into_box(self) -> Box<Self> {
        Box::new(self)
    }

    #[inline(always)]
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error>
    where
        Self: Serialize
    {
        serde_json::to_value(self)
    }

    #[inline(always)]
    fn to_json_string(&self) -> Result<String, serde_json::Error>
    where
        Self: Serialize
    {
        serde_json::to_string(self)
    }

    #[inline(always)]
    fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error>
    where
        Self: DeserializeOwned
    {
        serde_json::from_value(value)
    }

    #[inline(always)]
    fn from_json_str<'a>(value: &'a str) -> Result<Self, serde_json::Error>
    where
        Self: Deserialize<'a>
    {
        serde_json::from_str(value)
    }

    #[inline(always)]
    fn from_json_slice<'a>(value: &'a [u8]) -> Result<Self, serde_json::Error>
    where
        Self: Deserialize<'a>
    {
        serde_json::from_slice(value)
    }
}
