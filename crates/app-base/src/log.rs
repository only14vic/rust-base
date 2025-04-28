use {
    crate::{
        alloc::string::ToString,
        base::{ok, Void}
    },
    alloc::{format, string::String},
    core::{
        ffi::{c_char, c_int, CStr},
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

/// Logging levels for C
#[repr(C)]
#[allow(unused)]
#[allow(clippy::upper_case_acronyms)]
enum LogLevel {
    ERROR = 1, // Level::Error
    WARN = 2,  // Level::Warn
    INFO = 3,  // Level::Info
    DEBUG = 4, // Level::Debug
    TRACE = 5  // Level::Trace
}

impl Into<Level> for LogLevel {
    fn into(self) -> Level {
        unsafe { transmute(self as usize) }
    }
}

/// Initializes logging
///
/// Returns zero if initialization is successfull.
/// Otherwise returns -1.
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

/// Logs messages in C
#[no_mangle]
unsafe extern "C" fn log_msg(level: LogLevel, msg: *const c_char) {
    let msg = CStr::from_ptr(msg.cast()).to_string_lossy();
    log::log!(level.into(), "{msg}");
}

pub struct Logger;

impl Logger {
    pub fn init() -> Void {
        let level: LevelFilter = unsafe {
            match getenv(c"LOG_LEVEL".as_ptr()) {
                level if level.is_null() == false => {
                    let level = CStr::from_ptr(level).to_string_lossy();
                    let level = level.trim();
                    if level.is_empty() {
                        LEVEL_DEFAULT
                    } else {
                        level.parse().map_err(|e: ParseLevelError| e.to_string())?
                    }
                },
                _ => LEVEL_DEFAULT
            }
        };

        log::set_logger(&LOGGER).map_err(|e| e.to_string())?;
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
                l @ Level::Debug => l.bright_blue().to_string()
            };
            eprintln!(
                "{:<16} [{}] {}",
                format!("[{level_colored}]"),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
