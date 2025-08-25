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
    fn load_args(&mut self, args: &Args) -> Void;
}

type ArgsOpts = IndexMap<&'static str, &'static [&'static str]>;
type ArgsMap = IndexMap<String, Option<String>>;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum ArgUndefined {
    Skip,
    Allow,
    #[default]
    Error
}

impl FromStr for ArgUndefined {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "skip" => Ok(Self::Skip),
            "allow" => Ok(Self::Allow),
            "error" => Ok(Self::Error),
            s => Err(format!("Invalid value '{s}' of type ArgUndefinedBehavior."))
        }
    }
}

#[derive(Debug, Default)]
pub struct Args {
    pub opts: ArgsOpts,
    pub args: ArgsMap,
    pub undefined: ArgUndefined
}

impl Deref for Args {
    type Target = ArgsMap;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl Args {
    pub fn new(
        opts: impl IntoIterator<Item = (&'static str, &'static [&'static str], Option<&'static str>)>
    ) -> Ok<Self> {
        let mut args = Self::default();
        args.add_options(opts)?;
        Ok(args)
    }

    pub fn set_undefined(&mut self, behavior: ArgUndefined) -> &mut Self {
        self.undefined = behavior;
        self
    }

    pub fn add_options(
        &mut self,
        opts: impl IntoIterator<Item = (&'static str, &'static [&'static str], Option<&'static str>)>
    ) -> Ok<&mut Self> {
        for (n, o, v) in opts {
            if self.opts.contains_key(n) {
                Err(format!("Not unique option: {n}"))?;
            }
            if let Some(o) = self
                .opts
                .iter()
                .find_map(|(.., &ol)| ol.iter().find(|v| o.contains(v)))
            {
                Err(format!("Not unique option: {o}"))?;
            }

            self.opts.insert(n, o);
            self.args.insert(n.into(), v.map(|s| s.into()));
        }

        Ok(self)
    }

    pub unsafe fn parse_argc(&mut self, argc: c_int, argv: *const *const c_char) -> Ok<&mut Self> {
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
                {
                    i += 1;
                    Some(val)
                } else {
                    None
                }
            } else {
                None
            };

            if arg.starts_with("--") {
                if let Some((arg, val)) = arg.split_once("=") {
                    self.args.insert(self.arg_name(arg)?, val.into_some());
                } else if let Some(val) = next_val {
                    self.args.insert(self.arg_name(arg)?, val.into_some());
                } else {
                    self.args.insert(self.arg_name(arg)?, "1".into_some());
                }
            } else if arg.starts_with("-") {
                let last = arg.chars().last().unwrap();
                for ch in arg.chars().skip(1) {
                    if ch == last && next_val.is_some() {
                        self.args.insert(
                            self.arg_name(&['-', ch].iter().collect::<String>())?,
                            next_val.map(|s| s.to_string())
                        );
                    } else {
                        self.args.insert(
                            self.arg_name(&['-', ch].iter().collect::<String>())?,
                            "1".into_some()
                        );
                    }
                }
            } else {
                self.args
                    .insert(self.arg_name(&n.to_string())?, arg.into_some());
                n += 1;
            }
        }

        if self.undefined == ArgUndefined::Skip {
            self.args.shift_remove("");
        }

        self.into_ok()
    }

    pub fn get_option(&self, name: &str) -> Option<&str> {
        match self.args.get(name) {
            Some(v) => v.as_ref().map(String::as_str),
            None => {
                panic!("Undefined option name of command line argument: {name}")
            }
        }
    }

    fn arg_name(&self, arg: &str) -> Result<String, String> {
        self.opts
            .iter()
            .find(|(n, v)| {
                **n == arg
                    || v.contains(&arg)
                    || arg.get(0..2) == Some("--") && arg.get(2..) == Some(n)
            })
            .map(|(&n, _)| n.into_ok())
            .unwrap_or_else(|| {
                if self.opts.is_empty() || arg == "0" || self.undefined == ArgUndefined::Allow {
                    arg.into_ok()
                } else if self.undefined == ArgUndefined::Skip {
                    "".into_ok()
                } else {
                    Err(format!("Invalid command line argument: {arg}"))
                }
            })
    }
}
