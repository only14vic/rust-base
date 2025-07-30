#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
extern crate core;

#[cfg(not(feature = "std"))]
use core::ffi::{c_char, c_int};

#[allow(unused_imports)]
use {app::*, app_base::prelude::*};

#[cfg(feature = "std")]
fn main() -> Void {
    let app = App::boot().inspect_err(|e| log::error!("{e}"))?;
    run(app)
}

#[cfg(not(feature = "std"))]
#[unsafe(no_mangle)]
fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    let app = App::boot(argc, argv)
        .inspect_err(|e| panic!("{e}"))
        .unwrap();

    run(app).inspect_err(|e| panic!("{e}")).unwrap();

    libc::EXIT_SUCCESS
}

fn run(app: App) -> Void {
    let config = app.config();
    let args = app.get_ref::<Args<'_>>().unwrap();
    let cmd = args.get("command").ok_or("Undefined argument 'command'")?;
    let name = args.get("value").ok_or("Undefined argument 'value'")?;
    let exe = args
        .get("exe")
        .ok_or("Undefined argument 'exe'")?
        .as_ref()
        .map(|s| {
            let mut s = s.clone();
            s.replace_range(0..=s.rfind("/").unwrap_or(0), "");
            s
        })
        .ok_or("Argument 'exe' must be defined")?;

    match cmd.as_ref().map(|c| c.as_str()) {
        Some("show") => {
            for (k, v) in config.iter() {
                println!("{k}={v}");
            }
        },
        Some("get") => {
            let name = name
                .as_ref()
                .ok_or("Define config option name as argument")?;
            if let Some((.., value)) = config.iter().find(|(k, _)| *k == name.as_str()) {
                println!("{value}");
            } else {
                Err(format!("Invalid config option name: {name}"))?;
            }
        },
        None | Some("help") => {
            println!(
                r#"
Usage: {exe} <command> [value] [options]

Commands:
    show - list all config options
    get  - get config option value
    help - show usage help
"#,
            );
        },
        Some(cmd) => {
            Err(format!("Invalid command name: {cmd}"))?;
        }
    }

    ok()
}
