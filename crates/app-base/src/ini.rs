#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use libc_print::std_name::*;
use {
    crate::{
        base::{BaseFromInto, Ok},
        binds
    },
    alloc::{boxed::Box, ffi::CString, string::String, vec::Vec},
    core::{
        error::Error,
        ffi::{c_char, c_int, c_void, CStr},
        fmt::Display,
        ops::Deref,
        str::FromStr
    }
};

type IniMap = crate::base::IndexMap<Box<str>, Option<Box<str>>>;

#[derive(Debug)]
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

#[derive(Debug, Clone)]
pub struct Ini {
    items: IniMap
}

impl Deref for Ini {
    type Target = IniMap;

    fn deref(&self) -> &Self::Target {
        &self.items
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

        let c_path = &[path.as_ref().as_bytes(), b"\0"].concat();
        let c_path = CStr::from_bytes_with_nul(c_path)?;

        unsafe {
            if libc::access(c_path.as_ptr().cast(), libc::F_OK) != 0 {
                Err(IniError::FileNotFound(
                    ["Ini file not found: ", path.as_ref()].concat()
                ))?;
            }

            if binds::ini_parse(
                c_path.as_ptr().cast(),
                Some(Self::ini_parse_callback),
                (&mut this.items as *mut IniMap).cast()
            ) != 0
            {
                Err(IniError::InvalidParse(
                    ["Couldn't parse Ini file: ", path.as_ref()].concat()
                ))?;
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
                    libc::setenv(
                        name.as_ptr().cast(),
                        value.as_ptr().cast(),
                        overwrite.into()
                    );
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

    pub fn dotenv(overwrite: bool) -> Ok<Self> {
        Self::setenv_from_file(&".env", overwrite)
    }

    unsafe extern "C" fn ini_parse_callback(
        user: *mut c_void,
        section: *const c_char,
        name: *const c_char,
        value: *const c_char
    ) -> c_int {
        let items: &mut IniMap = &mut *user.cast();
        let section = CStr::from_ptr(section);
        let name = CStr::from_ptr(name);
        let value = CStr::from_ptr(value);

        let key: String = if section.is_empty() {
            name.to_string_lossy().into_owned()
        } else {
            section.to_string_lossy().into_owned() + "." + &name.to_string_lossy()
        };

        let mut value: String = value.to_string_lossy().into_owned();

        if let (Some(fc), Some(lc)) = (value.chars().next(), value.chars().last()) {
            if ['\'', '\"'].contains(&fc) && fc == lc && value.chars().count() > 1 {
                value = value.trim_matches(fc).into();
            };
        }

        items.insert(
            key.into(),
            if value.is_empty() { None } else { Some(value.into()) }
        );

        return 1;
    }
}

#[no_mangle]
extern "C" fn dotenv(overwrite: bool) -> c_int {
    match Ini::dotenv(overwrite) {
        Ok(..) => 0,
        Err(e) if e.downcast_ref::<IniError>().is_some() => {
            match e.downcast_ref::<IniError>() {
                // don't panic if file not exists
                Some(IniError::FileNotFound(..)) => -1,
                Some(e) => panic!("ERROR: {e}"),
                None => unreachable!()
            }
        },
        Err(..) => -2
    }
}
