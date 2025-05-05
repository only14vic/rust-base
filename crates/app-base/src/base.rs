use {
    ahash::AHasher,
    alloc::{
        boxed::Box,
        format,
        rc::Rc,
        string::{String, ToString},
        sync::Arc
    },
    core::{
        any::type_name,
        error::Error,
        fmt::{self, Debug, Display},
        hash::BuildHasherDefault,
        ops::Deref
    },
    serde::{Deserialize, Serialize, de::DeserializeOwned}
};

pub type IndexMap<K, V, S = BuildHasherDefault<AHasher>> = indexmap::IndexMap<K, V, S>;
pub type IndexSet<V, S = BuildHasherDefault<AHasher>> = indexmap::IndexSet<V, S>;

#[derive(PartialEq, Eq)]
pub struct ErrBox<E: ?Sized>(pub Box<E>);

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
        match error.downcast::<Box<Err>>() {
            Ok(e) => **e,
            Err(e) => {
                match e.downcast::<Box<ErrAsync>>() {
                    Ok(e) => Self::from(**e),
                    Err(e) => Self(e)
                }
            },
        }
    }
}
impl ErrAsync {
    #[inline(always)]
    pub fn new(error: Box<dyn Error + Send + Sync>) -> Self {
        match error.downcast::<Box<ErrAsync>>() {
            Ok(e) => **e,
            Err(e) => {
                match e.downcast::<Box<Err>>() {
                    Ok(e) => Self::from(**e),
                    Err(e) => Self(e)
                }
            },
        }
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
        write!(f, "{}", &self.0)
    }
}
impl Debug for ErrAsync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Display for Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
impl Display for ErrAsync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
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

pub trait TryMut {
    type Inner;

    fn try_mut(&mut self) -> Ok<&mut Self::Inner>;
}

impl<T> TryMut for Arc<T> {
    type Inner = T;

    #[inline]
    fn try_mut(&mut self) -> Ok<&mut Self::Inner> {
        Arc::get_mut(self)
            .ok_or_else(|| {
                format!("Could not get mutable reference of {}", type_name::<Self>())
            })?
            .into_ok()
    }
}

impl<T> TryMut for Rc<T> {
    type Inner = T;

    #[inline]
    fn try_mut(&mut self) -> Ok<&mut Self::Inner> {
        Rc::get_mut(self)
            .ok_or_else(|| {
                format!("Could not get mutable reference of {}", type_name::<Self>())
            })?
            .into_ok()
    }
}

pub trait Iter<'a, I: 'a> {
    fn iter(&'a self) -> impl Iterator<Item = I>;
}

pub trait IterMut<'a, I: 'a> {
    fn iter_mut(&'a mut self) -> impl Iterator<Item = I>;
}
