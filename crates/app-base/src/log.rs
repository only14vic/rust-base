use {
    crate::{
        alloc::string::ToString,
        base::{ok, Void}
    },
    alloc::string::String,
    core::{
        ffi::{c_char, c_int, c_uchar, CStr},
        mem::transmute
    },
    libc::getenv,
    log::{Level, LevelFilter, Log, ParseLevelError},
    yansi::Paint
};
#[cfg(not(feature = "std"))]
use libc_print::std_name::*;

static LOGGER: Logger = Logger;

#[cfg(debug_assertions)]
const LEVEL_DEFAULT: LevelFilter = LevelFilter::Debug;
#[cfg(not(debug_assertions))]
const LEVEL_DEFAULT: LevelFilter = LevelFilter::Info;

pub struct Logger;

#[no_mangle]
pub extern "C" fn log_init() -> c_int {
    match Logger::init() {
        Ok(..) => 0,
        Err(e) => {
            eprintln!("ERROR: log_init() - {e}");
            -1
        }
    }
}

#[no_mangle]
extern "C" fn log_msg(level: c_uchar, msg: *const c_char) -> c_int {
    if level < 1 || level > 5 {
        return -1;
    }

    let level: Level = unsafe { transmute(level as usize) };
    let msg = unsafe { CStr::from_ptr(msg.cast()).to_string_lossy() };

    log::log!(level, "{msg}");

    0
}

impl Logger {
    pub fn init() -> Void {
        let level: LevelFilter = unsafe {
            match getenv(c"LOG_LEVEL".as_ptr()) {
                level if level.is_null() == false => {
                    let level = CStr::from_ptr(level).to_string_lossy();
                    if level.trim().is_empty() {
                        LEVEL_DEFAULT
                    } else {
                        level
                            .trim()
                            .parse()
                            .map_err(|e: ParseLevelError| e.to_string())?
                    }
                },
                _ => LEVEL_DEFAULT
            }
        };

        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(level);
        log::debug!("LOG_LEVEL: {level}");

        ok()
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let level_colored: String = match record.level() {
                l @ Level::Info => l.bright_green().to_string(),
                l @ Level::Warn => l.bright_yellow().to_string(),
                l @ Level::Error => l.bright_red().to_string(),
                l @ Level::Trace => l.bright_black().to_string(),
                l @ Level::Debug => l.bright_magenta().to_string()
            };
            eprintln!(
                "[{}] [{}] {}",
                level_colored,
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
