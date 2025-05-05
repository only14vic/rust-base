use {
    crate::prelude::*,
    alloc::{
        format, slice,
        string::{String, ToString},
        vec::Vec
    },
    core::{
        ffi::{CStr, c_char, c_int},
        ops::Deref,
        str::{self, FromStr}
    }
};

pub trait LoadArgs {
    fn init_args(&mut self, args: &mut Args);

    fn load_args(&mut self, args: &Args);
}

type ArgsOpt = IndexMap<&'static str, Option<&'static str>>;
type ArgsMap = IndexMap<String, Option<String>>;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum ArgUndef {
    Skip,
    Add,
    #[default]
    Error
}

impl FromStr for ArgUndef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "skip" => Ok(Self::Skip),
            "allow" => Ok(Self::Add),
            "error" => Ok(Self::Error),
            s => Err(format!("Invalid value '{s}' of type ArgUndefinedBehavior."))
        }
    }
}

#[derive(Debug, Default)]
pub struct Args {
    pub opts: ArgsOpt,
    pub args: ArgsMap,
    pub undefined: ArgUndef
}

impl Deref for Args {
    type Target = ArgsMap;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl Args {
    pub const TYPE_BOOL: &str = ":b";

    pub fn new(
        opts: impl IntoIterator<
            Item = (&'static str, Option<&'static str>, Option<&'static str>)
        >
    ) -> Ok<Self> {
        let mut args = Self::default();
        args.add_options(opts)?;
        Ok(args)
    }

    pub fn set_undefined(&mut self, behavior: ArgUndef) -> &mut Self {
        self.undefined = behavior;
        self
    }

    /// Add command line options
    ///
    /// *opts* is a iter of tuple: (<long name>, <short name>, <default value>)
    ///
    /// Use number as short name option to determine argument of command line.
    pub fn add_options(
        &mut self,
        opts: impl IntoIterator<
            Item = (&'static str, Option<&'static str>, Option<&'static str>)
        >
    ) -> Ok<&mut Self> {
        for (n, o, v) in opts {
            if self.opts.contains_key(n) {
                Err(format!("Not unique option: {n}"))?;
            }
            if o.is_some() && self.opts.iter().any(|(_, v)| *v == o) {
                Err(format!("Not unique option: {}", o.unwrap()))?;
            }

            self.opts.insert(n, o);
            let n = n.split(':').next().unwrap();
            self.args.insert(n.into(), v.map(|v| v.into()));
        }

        Ok(self)
    }

    pub unsafe fn parse_argc(
        &mut self,
        argc: c_int,
        argv: *const *const c_char
    ) -> Ok<&mut Self> {
        let mut args = Vec::with_capacity(argc as usize);

        for arg in unsafe { slice::from_raw_parts(argv, argc as usize) } {
            let arg = unsafe { CStr::from_ptr(*arg).to_str()?.to_string() };
            args.push(arg);
        }

        self.parse_args(args)
    }

    pub fn parse_args(&mut self, args: Vec<String>) -> Ok<&mut Self> {
        Env::is_debug().then(|| log::trace!("Parsing command line arguments: {args:?}"));

        let mut i = 0;
        let mut n = 0;

        while i < args.len() {
            let arg: &str = args[i].as_ref();
            i += 1;

            let next_val = if let Some(val) = args.get(i) {
                if val.starts_with("-") == false
                    && arg.starts_with("-")
                    && arg.contains("=") == false
                    && self.arg_name(arg).map(|a| {
                        self.opts
                            .contains_key([a, Self::TYPE_BOOL].concat().as_str())
                    }) != Ok(true)
                {
                    i += 1;
                    Some(val)
                } else {
                    None
                }
            } else {
                None
            };

            if arg == "-" || arg.starts_with("--") {
                if let Some((arg, val)) = arg.split_once("=") {
                    self.args
                        .insert(self.arg_name(arg)?.into(), val.into_some());
                } else if let Some(val) = next_val {
                    self.args
                        .insert(self.arg_name(arg)?.into(), val.into_some());
                } else {
                    self.args
                        .insert(self.arg_name(arg)?.into(), "1".into_some());
                }
            } else if arg.starts_with("-") {
                let last = arg.chars().last().unwrap();
                for ch in arg.chars().skip(1) {
                    if ch == last && next_val.is_some() {
                        self.args.insert(
                            self.arg_name(&['-', ch].iter().collect::<String>())?.into(),
                            next_val.map(|s| s.to_string())
                        );
                    } else {
                        self.args.insert(
                            self.arg_name(&['-', ch].iter().collect::<String>())?.into(),
                            "1".into_some()
                        );
                    }
                }
            } else {
                self.args.insert(
                    self.arg_name(&n.to_string())
                        .map_err(|e| e.replace(&format!("'{n}'"), &format!("'{arg}'")))?
                        .into(),
                    arg.into_some()
                );
                n += 1;
            }
        }

        if self.undefined == ArgUndef::Skip {
            self.args.shift_remove("");
        }

        self.into_ok()
    }

    pub fn get(&self, name: &str) -> Ok<Option<&str>> {
        self.args
            .iter()
            .find_map(|(n, v)| n.eq(name).then_some(v.as_ref().map(String::as_str)))
            .ok_or_else(|| {
                format!("Undefined option name of command line arguments: {name}")
            })?
            .into_ok()
    }

    pub fn get_flag(&self, name: &str) -> Ok<bool> {
        self.get(name).map(|v| v == Some("1"))
    }

    fn arg_name<'a>(&'a self, arg: &'a str) -> Result<&'a str, String> {
        let arg_name = arg.trim_start_matches("-");
        self.opts
            .iter()
            .find_map(|(n, o)| {
                let n = n.split(':').next().unwrap();
                (n == arg || n == arg_name || *o == Some(arg) || *o == Some(arg_name))
                    .then_some(n)
            })
            .or_else(|| {
                if arg == "0" || self.undefined == ArgUndef::Add {
                    Some(arg)
                } else if self.undefined == ArgUndef::Skip {
                    Some("")
                } else {
                    None
                }
            })
            .ok_or_else(|| format!("Invalid command line argument: '{arg}'"))
    }
}
