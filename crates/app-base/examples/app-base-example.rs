#![cfg_attr(not(feature = "std"), no_std)]
#![no_main]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
extern crate core;

use {
    alloc::{
        boxed::Box,
        string::{String, ToString},
        vec::Vec
    },
    app_base::prelude::*,
    core::{
        ffi::{c_char, c_int},
        hint::black_box,
        num::NonZero,
        str::FromStr,
        usize
    },
    serde::Serialize,
    yansi::Paint
};

const MAX_ITERS: usize = 100_000;
const FILE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/data.ini");

#[unsafe(no_mangle)]
fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    dotenv(false);
    log_init();

    let mut args = Args::new([
        ("self", "0".into(), None),
        ("command", "1".into(), None),
        ("subcommand", "2".into(), None),
        ("foo", "-f".into(), "Foo".into_some()),
        ("bar", "-b".into(), "Bar".into_some()),
        ("zoo", "-z".into(), None)
    ])
    .unwrap();
    unsafe { args.parse_argc(argc, argv).unwrap() };

    log::debug!("{:#?}", &*args);

    let mut config = Config::default();

    for _ in 0..MAX_ITERS {
        black_box({
            let ini = Ini::from_file(&FILE_PATH)
                .inspect_err(|e| log::error!("{e}"))
                .unwrap();

            config.extend(&ini);
        });
    }

    log::info!(
        "Struct {}",
        format!("{:#?}", &config).bright_green().italic()
    );

    let config_json = config.to_json().unwrap();
    log::debug!(
        "JSON {}",
        format!("{:#?}", &config_json).bright_cyan().italic()
    );

    log::info!("Max iters: {}", MAX_ITERS.bright_red().bold());
    log::trace!(
        "{}",
        format!("no_std = {}", cfg!(not(feature = "std")))
            .red()
            .on_green()
    );
    log::trace!(
        "{}",
        format!("static = {}", cfg!(target_env = "musl"))
            .blue()
            .on_bright_green()
    );

    mem_stats();

    libc::EXIT_SUCCESS
}

#[derive(Default, Debug, Serialize, Extend)]
pub struct Config {
    version: f32,
    general: General
}

#[derive(Default, Debug, PartialEq, Serialize)]
pub enum Lang {
    #[default]
    Ru,
    En
}

impl FromStr for Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ru" => Ok(Self::Ru),
            "en" => Ok(Self::En),
            _ => Err("Invalid value".to_string())
        }
    }
}

#[derive(Default, Debug, Serialize, Extend)]
pub struct General {
    #[parse]
    str: Option<Box<Lang>>,
    number: u32,
    boolean: bool,
    list: Option<Vec<u32>>,
    text: String,
    foo: Foo
}

#[derive(Default, Debug, Serialize, Extend)]
pub struct Foo {
    #[parse]
    str: Lang,
    number: Option<NonZero<u32>>,
    boolean: Option<bool>,
    text: Box<str>
}
