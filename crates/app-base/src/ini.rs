#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use libc_print::std_name::*;
use {
    crate::{
        base::{BaseFromInto, Ok},
        binds,
        prelude::Env
    },
    alloc::{boxed::Box, ffi::CString, format, string::String, vec::Vec},
    core::{
        error::Error,
        ffi::{CStr, c_char, c_int, c_void},
        fmt::Display,
        ops::{Deref, DerefMut},
        str::FromStr
    }
};

type IniMap = crate::base::IndexMap<Box<str>, Option<Box<str>>>;

#[derive(Debug, PartialEq, Eq)]
pub enum IniError {
    FileNotFound(String),
    InvalidParse(String)
}
impl Error for IniError {}

impl Display for IniError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::FileNotFound(s) => s,
            Self::InvalidParse(s) => s
        };
        write!(f, "{s}")
    }
}

#[derive(Default, Debug, Clone)]
pub struct Ini {
    items: IniMap
}

impl Deref for Ini {
    type Target = IniMap;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl DerefMut for Ini {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<'a> IntoIterator for &'a Ini {
    type Item = (&'a str, Option<&'a str>);
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref().map(|v| v.as_ref())))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl Ini {
    pub fn from_file(path: &dyn AsRef<str>) -> Ok<Self> {
        let mut this = Self { items: Default::default() };
        let c_path = CString::from_str(path.as_ref())?;

        unsafe {
            if libc::access(c_path.as_ptr(), libc::F_OK) != 0 {
                Err(IniError::FileNotFound(format!(
                    "File not found: {}",
                    path.as_ref()
                )))?;
            }

            if binds::ini_parse(
                c_path.as_ptr(),
                Some(Self::ini_parse_callback),
                (&mut this.items as *mut IniMap).cast()
            ) != 0
            {
                Err(IniError::InvalidParse(format!(
                    "Could not parse config file: {}",
                    path.as_ref()
                )))?;
            }
        }

        this.into_ok()
    }

    pub fn setenv(&self, overwrite: bool) -> Ok<&Self> {
        for (k, v) in self.iter() {
            if let Some(v) = v {
                let name = CString::from_str(k.as_ref())?;
                let value = CString::from_str(v.as_ref())?;
                unsafe {
                    libc::setenv(name.as_ptr(), value.as_ptr(), overwrite.into());
                }
            }
        }

        self.into_ok()
    }

    pub fn setenv_from_file(path: &dyn AsRef<str>, overwrite: bool) -> Ok<Self> {
        let ini = Self::from_file(path)?;
        ini.setenv(overwrite)?;
        ini.into_ok()
    }

    pub fn dotenv(path: &dyn AsRef<str>, overwrite: bool) -> Ok<Self> {
        Self::setenv_from_file(path, overwrite)
    }

    extern "C" fn ini_parse_callback(
        context: *mut c_void,
        section: *const c_char,
        name: *const c_char,
        value: *const c_char
    ) -> c_int {
        if context.is_null() || name.is_null() || value.is_null() || section.is_null() {
            return 0;
        }

        let items: &mut IniMap = unsafe { &mut *context.cast() };
        let section = unsafe { CStr::from_ptr(section) };
        let name = unsafe { CStr::from_ptr(name) };
        let value = unsafe { CStr::from_ptr(value) };

        let key: String = if section.is_empty() {
            name.to_string_lossy().into_owned()
        } else {
            section.to_string_lossy().into_owned() + "." + &name.to_string_lossy()
        };

        let mut value: String = value.to_string_lossy().into_owned();

        if let (Some(fc), Some(lc)) = (value.chars().next(), value.chars().last())
            && ['\'', '\"'].contains(&fc)
            && fc == lc
            && value.chars().count() > 1
        {
            value = value.trim_matches(fc).into();
        };

        items.insert(
            key.into(),
            if value.is_empty() { None } else { Some(value.into()) }
        );

        return 1;
    }
}

/// Loads .env file variables
///
/// Returns zero if initialization is successfull.
/// Otherwise returns int less zero.
#[unsafe(no_mangle)]
pub extern "C" fn dotenv(overwrite: bool) -> c_int {
    let res = match Ini::dotenv(&".env", overwrite) {
        Ok(..) => {
            unsafe { Env::reset() };
            0
        },
        Err(e) => {
            match e.downcast_ref::<IniError>() {
                // don't panic if file not exists
                Some(IniError::FileNotFound(..)) => -1,
                _ => panic!("dotenv error: {e}")
            }
        }
    };

    if Env::is_test() {
        match Ini::dotenv(&".env.test", true) {
            Ok(..) => unsafe { Env::reset() },
            Err(e) => {
                match e.downcast_ref::<IniError>() {
                    // don't panic if file not exists
                    Some(IniError::FileNotFound(..)) => (),
                    _ => panic!("dotenv.test error: {e}")
                }
            }
        }
    }

    res
}
