use {
    crate::prelude::*,
    alloc::{
        format, slice,
        string::{String, ToString},
        vec::Vec
    },
    core::{
        ffi::{c_char, CStr},
        ops::Deref,
        str
    }
};

pub trait LoadArgs {
    fn load_args(&mut self, args: &Args) -> Ok<&mut Self>;
}

type ArgsOpts<'o> = IndexMap<&'o str, Vec<&'o str>>;
type ArgsMap = IndexMap<String, Option<String>>;

#[derive(Debug, Default)]
pub struct Args<'o> {
    pub opts: ArgsOpts<'o>,
    pub args: ArgsMap
}

impl Deref for Args<'_> {
    type Target = ArgsMap;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl<'o> Args<'o> {
    pub fn new(
        opts: impl IntoIterator<Item = (&'o str, Vec<&'o str>, Option<&'o str>)>
    ) -> Self {
        Self::default().with_opts(opts)
    }

    pub fn with_opts(
        mut self,
        opts: impl IntoIterator<Item = (&'o str, Vec<&'o str>, Option<&'o str>)>
    ) -> Self {
        for (n, o, v) in opts.into_iter() {
            self.opts.insert(n, o);
            self.args.insert(n.into(), v.map(|s| s.into()));
        }
        self
    }

    pub unsafe fn parse_argc(self, argc: usize, argv: *const *const c_char) -> Ok<Self> {
        let mut args = Vec::with_capacity(argc);
        for arg in slice::from_raw_parts(argv, argc) {
            let arg = CStr::from_ptr(*arg).to_str()?.to_string();
            args.push(arg);
        }
        self.parse_args(args)
    }

    pub fn parse_args(mut self, args: Vec<String>) -> Ok<Self> {
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

        self.into_ok()
    }

    fn arg_name(&self, arg: &str) -> Result<String, String> {
        self.opts
            .iter()
            .find(|(&n, v)| {
                n == arg
                    || v.contains(&arg)
                    || arg.get(0..2) == Some("--") && arg.get(2..) == Some(n)
            })
            .map(|(&n, _)| n.into_ok())
            .unwrap_or_else(|| {
                if self.opts.is_empty() || arg == "0" {
                    arg.into_ok()
                } else {
                    Err(format!(
                        "Invalid command option or argument: {arg}"
                    ))
                }
            })
    }
}
