use {
    crate::prelude::*,
    alloc::{
        ffi::CString,
        format,
        string::{String, ToString},
        vec::Vec
    },
    core::{mem::ManuallyDrop, ptr::null_mut, str::FromStr},
    libc::{dirname, getcwd, readlink, realpath}
};

const PATH_MAX: usize = libc::PATH_MAX as usize;

#[derive(Debug, ExtendFromIter)]
pub struct Dirs {
    exe: String,
    prefix: String,
    suffix: String,
    home: String,
    pub user_config: String,
    pub bin: String,
    pub lib: String,
    pub config: String,
    pub data: String,
    pub cache: String,
    pub runtime: String,
    pub state: String,
    pub log: String,
    pub tmp: String
}

impl Default for Dirs {
    fn default() -> Self {
        let mut this = Self {
            exe: Self::exe_path().unwrap(),
            prefix: "".into(),
            suffix: "".into(),
            home: "~".into(),
            user_config: "~/.config".into(),
            bin: "bin".into(),
            lib: "lib".into(),
            config: "etc".into(),
            data: "share".into(),
            state: "var/lib".into(),
            cache: "var/cache".into(),
            runtime: "var/run".into(),
            log: "var/log".into(),
            tmp: "/tmp".into()
        };
        this.init();
        this
    }
}

impl Dirs {
    const CONFIG_DIR_ENV: &str = "CONFIG_DIR";

    fn init(&mut self) -> &mut Self {
        let (home, prefix, suffix) = (
            self.home.clone(),
            self.prefix.clone(),
            self.suffix.clone()
        );
        self.with_home(&home)
            .with_prefix(&prefix)
            .with_suffix(&suffix)
    }

    pub fn with_home(&mut self, home: &str) -> &mut Self {
        let old_home = match self.home.as_str() {
            "" => "".into(),
            _ => format!("{}/", self.home.trim_end_matches("/"))
        };
        let mut home = Self::normalize_path(home);
        home = match home.as_str() {
            "" => panic!("The path of home directory cannot be empty."),
            _ => format!("{home}/")
        };

        for dir in self.user_dirs() {
            if old_home.is_empty() == false && dir.find(&old_home) == Some(0) {
                *dir = dir.replacen(&old_home, &home, 1);
            } else if dir.contains("~") {
                *dir = dir.replacen("~", &home, 1);
            } else if dir.starts_with("/") == false {
                dir.insert_str(0, &home);
            }
            *dir = Self::normalize_path(dir);
        }

        for dir in self.prefixed_dirs() {
            if dir.contains("~") {
                *dir = dir.replacen("~", &home, 1);
            }
            *dir = Self::normalize_path(dir);
        }

        self.home = home;

        if self.home.is_empty() == false {
            self.home.remove(self.home.len() - 1);
        }

        self
    }

    pub fn with_prefix(&mut self, prefix: &str) -> &mut Self {
        let old_prefix = match self.prefix.as_str() {
            "" => "".into(),
            _ => format!("{}/", self.prefix.trim_end_matches("/"))
        };
        let home = self.home.clone();
        let mut prefix = if prefix.contains("~") {
            prefix.replacen("~", &home, 1)
        } else {
            prefix.to_string()
        };
        prefix = Self::normalize_path(&prefix);
        prefix = match prefix.as_str() {
            "" => "".into(),
            _ => format!("{prefix}/")
        };

        for dir in self.prefixed_dirs() {
            if dir.find(&home) == Some(0) {
                continue;
            }
            if old_prefix.is_empty() == false && dir.find(&old_prefix) == Some(0) {
                *dir = dir.replacen(&old_prefix, &prefix, 1);
            } else if dir.contains("~") {
                *dir = dir.replacen("~", &home, 1);
            } else if dir.starts_with("/") == false {
                dir.insert_str(0, &prefix);
            }
            *dir = Self::normalize_path(dir);
        }

        self.prefix = prefix;

        if self.prefix.is_empty() == false {
            self.prefix.remove(self.prefix.len() - 1);
        }

        self
    }

    pub fn with_suffix(&mut self, suffix: &str) -> &mut Self {
        let old_suffix = match self.suffix.as_str() {
            "" => "".into(),
            _ => format!("/{}", self.suffix.trim_start_matches("/"))
        };
        let suffix = match suffix {
            "" => "".into(),
            _ => format!("/{}", suffix.trim_start_matches("/"))
        };

        for dir in self.suffixed_dirs() {
            if old_suffix.is_empty() == false
                && dir.rfind(&old_suffix).map(|p| p + old_suffix.len()) == Some(dir.len())
            {
                *dir = dir.replacen(&old_suffix, &suffix, 1);
            } else {
                dir.push_str(&suffix);
            }
        }

        self.suffix = suffix;

        if self.suffix.is_empty() == false {
            self.suffix.remove(0);
        }

        self
    }

    fn user_dirs(&mut self) -> impl IntoIterator<Item = &mut String> {
        [&mut self.user_config]
    }

    fn prefixed_dirs(&mut self) -> impl IntoIterator<Item = &mut String> {
        [
            &mut self.lib, &mut self.bin, &mut self.config, &mut self.data,
            &mut self.cache, &mut self.runtime, &mut self.state, &mut self.log,
            &mut self.tmp
        ]
    }

    fn suffixed_dirs(&mut self) -> impl IntoIterator<Item = &mut String> {
        [
            &mut self.lib, &mut self.config, &mut self.user_config, &mut self.data,
            &mut self.cache, &mut self.runtime, &mut self.state, &mut self.log,
            &mut self.tmp
        ]
    }

    fn normalize_path(path: &str) -> String {
        let mut path = path.to_string();

        if path.contains("~") {
            path = path.replacen(
                "~",
                &getenv("HOME").expect("HOME environment variable is not set."),
                1
            );
        }

        if path.starts_with("./") || path == "." {
            path = path.replacen(".", &Self::cwd().unwrap(), 1);
        }

        while path.contains("//") {
            path = path.replace("//", "/");
        }

        path = path.trim_end_matches("/").to_string();

        path
    }

    fn exe_path() -> Ok<String> {
        let pid = unsafe { libc::getpid() };
        let path = format!("/proc/{pid}/exe");
        let path_c = CString::from_str(&path)?;
        let mut buf = Vec::with_capacity(PATH_MAX);
        let size = unsafe { readlink(path_c.as_ptr(), buf.as_mut_ptr(), PATH_MAX) };

        if size <= 0 {
            return Err(format!("Could not read link '{path}'"))?;
        }

        let link_path = unsafe {
            ManuallyDrop::new(String::from_raw_parts(
                buf.as_mut_ptr().cast(),
                size as usize,
                PATH_MAX
            ))
        };

        Ok(link_path.to_string())
    }

    pub fn dirname(path: &str) -> Ok<String> {
        let path_c = ManuallyDrop::new(CString::from_str(path)?);
        let dir_ptr =
            unsafe { dirname(realpath(path_c.as_ptr().cast_mut(), null_mut())) };

        if dir_ptr.is_null() {
            return Err("Could not get dirname.")?;
        }

        let dir = unsafe { CString::from_raw(dir_ptr).into_string()? };
        Ok(dir)
    }

    pub fn cwd() -> Ok<String> {
        let dir = unsafe {
            let cwd = getcwd(null_mut(), 0);

            if cwd.is_null() {
                return Err("Could not get current work directory.")?;
            }

            CString::from_raw(cwd.cast()).into_string()?
        };

        Ok(dir)
    }

    #[inline]
    pub fn exe(&self) -> &str {
        &self.exe
    }

    #[inline]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    #[inline]
    pub fn suffix(&self) -> &str {
        &self.suffix
    }

    #[inline]
    pub fn home(&self) -> &str {
        &self.home
    }
}

impl LoadEnv for Dirs {
    fn load_env(&mut self) -> Void {
        #[rustfmt::skip]
        self.extend(
            [
                ("config", getenv(Self::CONFIG_DIR_ENV)),
            ]
            .iter()
            .map(convert::tuple_option_string_to_str)
        );
        self.init();
        ok()
    }
}

impl LoadArgs for Dirs {
    fn load_args(&mut self, args: &Args) -> Void {
        self.extend(
            [
                ("home", args.get("home-dir")),
                ("config", args.get("config-dir")),
                ("user_config", args.get("user-config-dir"))
            ]
            .iter()
            .map(convert::tuple_option_option_string_to_str)
        );
        self.init();
        ok()
    }
}
