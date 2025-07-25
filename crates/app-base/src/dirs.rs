use {
    crate::prelude::*,
    alloc::{
        ffi::CString,
        format,
        string::{String, ToString},
        vec::Vec
    },
    core::{cell::RefCell, mem::ManuallyDrop, ptr::null_mut, str::FromStr},
    libc::{dirname, getcwd, readlink, realpath}
};

const PATH_MAX: usize = libc::PATH_MAX as usize;

#[derive(Debug, Clone, ExtendFromIter)]
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
            config: option_env!("CONFDIR")
                .unwrap_or("{prefix}/etc/{suffix}")
                .into(),
            home: "~".into(),
            user_config: "{home}/.config/{suffix}".into(),
            bin: "{prefix}/bin".into(),
            sbin: "{prefix}/sbin".into(),
            lib: "{prefix}/lib".into(),
            include: "{prefix}/include/{suffix}".into(),
            man: "{prefix}/share/man/{suffix}".into(),
            doc: "{prefix}/share/doc/{suffix}".into(),
            state: "{var}/lib/{suffix}".into(),
            cache: "{var}/cache/{suffix}".into(),
            run: "{var}/run/{suffix}".into(),
            log: "{var}/log/{suffix}".into(),
            tmp: "/tmp/{suffix}".into()
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
}

impl LoadEnv for Dirs {
    fn load_env(&mut self) -> Void {
        #[rustfmt::skip]
        self.extend(
            [
                ("config", getenv("CONFIG_DIR")),
            ]
            .iter()
            .map(convert::tuple_option_string_to_str)
        );
        ok()
    }
}

impl LoadArgs for Dirs {
    fn load_args(&mut self, args: &Args) -> Void {
        self.extend(
            [
                ("home", args.get("home-dir")),
                ("config", args.get("config-dir")),
                ("user_config", args.get("user-config-dir")),
                ("log", args.get("log-dir"))
            ]
            .iter()
            .map(convert::tuple_option_option_string_to_str)
        );
        ok()
    }
}
