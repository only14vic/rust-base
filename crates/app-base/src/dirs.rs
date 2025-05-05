use {
    crate::prelude::*,
    alloc::{
        boxed::Box,
        ffi::CString,
        format,
        string::{String, ToString},
        vec::Vec
    },
    core::{cell::RefCell, fmt::Display, mem::ManuallyDrop, ptr::null_mut, str::FromStr},
    libc::{getcwd, readlink},
    serde::{Deserialize, Serialize}
};

const PATH_MAX: usize = libc::PATH_MAX as usize;

pub trait LoadDirs {
    fn load_dirs<'a>(&'a mut self, dirs: &'a Dirs);
}

#[derive(Debug, Clone, ExtendFromIter, Serialize, Deserialize)]
pub struct Dirs {
    exe: String,
    pub prefix: String,
    pub suffix: String,
    pub home: String,
    pub config: String,
    pub user_config: String,
    pub bin: String,
    pub sbin: String,
    pub lib: String,
    pub include: String,
    pub data: String,
    pub man: String,
    pub doc: String,
    pub var: String,
    pub cache: String,
    pub run: String,
    pub state: String,
    pub log: String,
    pub tmp: String
}

impl Default for Dirs {
    fn default() -> Self {
        Self {
            exe: Self::exe_path().unwrap(),
            prefix: option_env!("PREFIX").unwrap_or(".").into(),
            suffix: option_env!("SUFFIX").unwrap_or("").into(),
            var: option_env!("VARDIR").unwrap_or("{prefix}/var").into(),
            data: option_env!("DATADIR")
                .unwrap_or("{prefix}/share/{suffix}")
                .into(),
            config: option_env!("CONFDIR").unwrap_or("/etc/{suffix}").into(),
            home: "~".into(),
            user_config: option_env!("USERCONFDIR")
                .unwrap_or("{home}/.config/{suffix}")
                .into(),
            bin: option_env!("BINDIR").unwrap_or("{prefix}/bin").into(),
            sbin: option_env!("SBINDIR").unwrap_or("{prefix}/sbin").into(),
            lib: option_env!("LIBDIR").unwrap_or("{prefix}/lib").into(),
            include: option_env!("INCDIR")
                .unwrap_or("{prefix}/include/{suffix}")
                .into(),
            man: option_env!("MANDIR")
                .unwrap_or("{prefix}/share/man/{suffix}")
                .into(),
            doc: option_env!("DOCDIR")
                .unwrap_or("{prefix}/share/doc/{suffix}")
                .into(),
            state: option_env!("STATEDIR")
                .unwrap_or("{var}/lib/{suffix}")
                .into(),
            cache: option_env!("CACHEDIR")
                .unwrap_or("{var}/cache/{suffix}")
                .into(),
            run: option_env!("RUNDIR").unwrap_or("{var}/run/{suffix}").into(),
            log: option_env!("LOGDIR").unwrap_or("{var}/log/{suffix}").into(),
            tmp: option_env!("TEMPDIR").unwrap_or("/tmp/{suffix}").into()
        }
    }
}

impl Dirs {
    pub fn init(&mut self) -> &mut Self {
        let list: IndexMap<_, _> = IndexMap::from_iter([
            ("{prefix}", RefCell::new(&mut self.prefix)),
            ("{suffix}", (&mut self.suffix).into()),
            ("{var}", (&mut self.var).into()),
            ("{config}", (&mut self.config).into()),
            ("{home}", (&mut self.home).into()),
            ("{user_config}", (&mut self.user_config).into()),
            ("{bin}", (&mut self.bin).into()),
            ("{sbin}", (&mut self.sbin).into()),
            ("{lib}", (&mut self.lib).into()),
            ("{include}", (&mut self.include).into()),
            ("{data}", (&mut self.data).into()),
            ("{state}", (&mut self.state).into()),
            ("{cache}", (&mut self.cache).into()),
            ("{run}", (&mut self.run).into()),
            ("{log}", (&mut self.log).into()),
            ("{man}", (&mut self.man).into()),
            ("{doc}", (&mut self.doc).into()),
            ("{tmp}", (&mut self.tmp).into())
        ]);

        loop {
            let mut need_repeat = false;

            for (.., dir) in list.iter() {
                let mut dir = dir.borrow_mut();
                Self::normalize_path(*dir);

                if dir.contains('{') && dir.contains('}') {
                    for (name, subdir) in list.iter() {
                        if let Ok(mut subdir) = subdir.try_borrow_mut() {
                            if let Some(pos) = dir.find(name) {
                                Self::normalize_path(*subdir);
                                dir.replace_range(pos..pos + name.len(), *subdir);
                            }

                            if let Some((pos1, pos2)) =
                                subdir.find('{').zip(subdir.find('}'))
                                && pos2 > pos1
                            {
                                need_repeat =
                                    list.contains_key(subdir.get(pos1..=pos2).unwrap());
                            }
                        }
                    }
                }
            }

            if need_repeat == false {
                break;
            }
        }

        self
    }

    pub fn normalize_path(path: &mut String) {
        if let Some(pos) = path.find("~") {
            path.replace_range(
                pos..pos + 1,
                &getenv("HOME").expect("HOME environment variable is not set.")
            );
        }

        if path.starts_with("./") || path == "." {
            path.replace_range(0..1, &Self::cwd().unwrap());
        }

        while let Some(pos) = path.find("//") {
            path.remove(pos);
        }

        while let Some(pos) = path.rfind("/")
            && pos == path.len() - 1
        {
            path.remove(pos);
        }
    }

    pub fn dirname(path: &str) -> &str {
        if path.contains("/") {
            &path[0..path.rfind('/').unwrap()]
        } else {
            "."
        }
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

    #[inline]
    pub fn exe(&self) -> &str {
        &self.exe
    }

    #[inline]
    pub fn exe_file(&self) -> &str {
        self.exe
            .get(self.exe.rfind("/").map(|p| p + 1).unwrap_or(0)..self.exe.len())
            .unwrap()
    }

    #[cfg(feature = "std")]
    pub fn mkdir(path: &str) -> Void {
        let path = path.trim();
        if path.is_empty() || path == "." {
            return ok();
        }
        match std::fs::create_dir_all(path) {
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => ok(),
            Err(e) => Err(format!("Could not create directory - {e}: {path}"))?,
            Ok(..) => ok()
        }
    }

    #[cfg(not(feature = "std"))]
    pub fn mkdir(path: &str) -> Void {
        let path = path.trim();
        if path.is_empty() || path == "." {
            return ok();
        }
        unsafe {
            let c_dir = CString::new(path)?;
            if libc::access(c_dir.as_ptr(), libc::F_OK) != 0 {
                if libc::mkdir(c_dir.as_ptr(), 0o755) != 0 {
                    Err(format!("Could not create directory: {}", c_dir.to_str()?))?;
                }
            }
        }
        ok()
    }
}

impl Iter<'_, (&'static str, String)> for Dirs {
    fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [
            ("dirs.exe", Box::leak(Box::new(self.exe())) as &dyn Display),
            ("dirs.bin", &self.bin),
            ("dirs.sbin", &self.sbin),
            ("dirs.lib", &self.lib),
            ("dirs.man", &self.man),
            ("dirs.doc", &self.doc),
            ("dirs.var", &self.var),
            ("dirs.run", &self.run),
            ("dirs.log", &self.log),
            ("dirs.data", &self.data),
            ("dirs.cache", &self.cache),
            ("dirs.state", &self.state),
            ("dirs.config", &self.config),
            ("dirs.user_config", &self.user_config),
            ("dirs.home", &self.home),
            ("dirs.include", &self.include),
            ("dirs.tmp", &self.tmp),
            ("dirs.prefix", &self.prefix),
            ("dirs.suffix", &self.suffix)
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
    }
}

impl LoadArgs for Dirs {
    fn init_args(&mut self, args: &mut Args) {
        args.add_options([
            ("dirs-home", None, None),
            ("dirs-config", None, None),
            ("dirs-user-config", None, None),
            ("dirs-bin", None, None),
            ("dirs-sbin", None, None),
            ("dirs-lib", None, None),
            ("dirs-log", None, None),
            ("dirs-var", None, None),
            ("dirs-run", None, None),
            ("dirs-data", None, None),
            ("dirs-cache", None, None),
            ("dirs-state", None, None),
            ("dirs-tmp", None, None)
        ])
        .unwrap();
    }

    fn load_args(&mut self, args: &Args) {
        self.extend(
            [
                ("home", args.get("dirs-home")),
                ("config", args.get("dirs-config")),
                ("user_config", args.get("dirs-user-config")),
                ("bin", args.get("dirs-bin")),
                ("sbin", args.get("dirs-sbin")),
                ("lib", args.get("dirs-lib")),
                ("log", args.get("dirs-log")),
                ("var", args.get("dirs-var")),
                ("run", args.get("dirs-run")),
                ("data", args.get("dirs-data")),
                ("cache", args.get("dirs-cache")),
                ("state", args.get("dirs-state")),
                ("tmp", args.get("dirs-tmp"))
            ]
            .iter()
            .map(convert::tuple_result_option_str)
        );
    }
}

impl LoadEnv for Dirs {
    fn load_env(&mut self) {}
}
