use {
    crate::{
        alloc::{ffi::CString, string::ToString},
        prelude::*
    },
    alloc::{boxed::Box, format, string::String},
    core::{
        ffi::{c_char, CStr},
        mem::{transmute, zeroed},
        ops::{Deref, DerefMut},
        ptr::null_mut,
        str::FromStr,
        sync::atomic::{AtomicBool, Ordering}
    },
    log::{Level, Log},
    yansi::Paint
};
#[cfg(not(feature = "std"))]
use libc_print::std_name::*;

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
/// Returns non-zero pointer if initialization is successfull.
/// Otherwise returns zero.
#[no_mangle]
pub extern "C" fn log_init() -> *mut Logger {
    match Logger::init() {
        Ok(logger) => Box::into_raw(logger),
        Err(e) => {
            eprintln!("ERROR: log_init() - {e}");
            null_mut()
        }
    }
}

/// Logs messages in C
#[no_mangle]
unsafe extern "C" fn log_msg(level: LogLevel, msg: *const c_char) {
    let msg = CStr::from_ptr(msg.cast()).to_string_lossy();
    log::log!(level.into(), "{msg}");
}

/// Logger
#[derive(Default)]
pub struct Logger {
    config: LogConfig,
    file: Option<Box<libc::FILE>>,
    lock: AtomicBool
}

impl Deref for Logger {
    type Target = LogConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl DerefMut for Logger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

impl Logger {
    pub fn init() -> Ok<Box<Self>> {
        let mut logger = Box::new(Self::default());
        let logger_ref: &'static Self = unsafe { &*(logger.as_ref() as *const _) };

        log::set_logger(logger_ref).map_err(|e| e.to_string())?;

        logger.load_env()?;
        logger.configure(&logger_ref.config)?;

        Ok(logger)
    }

    pub fn configure(&mut self, config: &LogConfig) -> Void {
        self.log_close();
        self.config = config.clone();

        if let Some(path) = self.config.file.as_ref() {
            if path.is_empty() == false {
                unsafe {
                    let file = libc::fopen(
                        CString::from_str(path.as_str())?.as_ptr(),
                        c"a+".as_ptr()
                    );
                    if file.is_null() {
                        Err("Could not open log file: {path}")?;
                    }
                    self.file = Box::from_raw(file).into();
                }
            }
        }

        log::set_max_level(self.config.level);
        log::trace!("{:?}", self.config);

        ok()
    }

    /// Close log file descriptor
    #[no_mangle]
    extern "C" fn log_close(&mut self) {
        if let Some(file) = self.file.take() {
            unsafe { libc::fclose(Box::into_raw(file)) };

            log::trace!(
                "LOG FILE CLOSED: {}",
                self.config
                    .file
                    .as_ref()
                    .unwrap_or(&String::with_capacity(0))
            );
        }
    }

    fn time() -> String {
        unsafe {
            let mut time: libc::timeval = zeroed();
            libc::gettimeofday(&mut time as *mut _, null_mut());
            let local = &*libc::localtime(&time.tv_sec);
            const BUFF_LEN: usize = 60;
            let buff = &mut [0u8; BUFF_LEN] as *mut _ as *mut i8;

            libc::strftime(buff, BUFF_LEN, c"%F %T".as_ptr(), local);
            libc::sprintf(
                buff.wrapping_add(libc::strlen(buff)),
                c".%06ld".as_ptr(),
                time.tv_usec
            );
            libc::strftime(
                buff.wrapping_add(libc::strlen(buff)),
                6,
                c"%z".as_ptr(),
                local
            );

            CStr::from_ptr(buff).to_string_lossy().to_string()
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.log_close();
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) == false {
            return;
        }

        if let Some(filter) = self.config.filter.as_ref() {
            let target = record.target();
            for value in filter.iter() {
                if let Some(value) = value.strip_prefix("!") {
                    if target.starts_with(value) {
                        return;
                    }
                } else if target.starts_with(value) {
                    break;
                }
            }
        }

        let allow_color = self.config.color && self.file.is_none();
        let level = if allow_color {
            match record.level() {
                l @ Level::Info => l.bright_green().to_string(),
                l @ Level::Warn => l.bright_yellow().to_string(),
                l @ Level::Error => l.bright_red().to_string(),
                l @ Level::Trace => l.bright_black().to_string(),
                l @ Level::Debug => l.bright_blue().to_string()
            }
        } else {
            record.level().to_string()
        };
        let out = format!(
            "[{}] [pid:{} tid:{}] {:<len$} [{}] {}\n",
            Self::time(),
            unsafe { libc::getpid() },
            unsafe { libc::pthread_self() as usize },
            format!("[{}]", level),
            record.target(),
            record.args(),
            len = if allow_color { 16 } else { 7 }
        );

        if self.file.is_none() {
            eprint!("{out}");
            return;
        }

        while self.lock.swap(true, Ordering::SeqCst) {
            #[cfg(not(feature = "std"))]
            unsafe {
                libc::sched_yield();
            }
            #[cfg(feature = "std")]
            std::thread::yield_now();
        }

        unsafe {
            libc::fputs(
                CString::from_str(&out)
                    .inspect_err(|_| self.lock.store(false, Ordering::SeqCst))
                    .unwrap()
                    .as_ptr(),
                self.file.as_ref().unwrap().as_ref() as *const _ as *mut _
            );
        }

        self.lock.store(false, Ordering::SeqCst);
    }

    fn flush(&self) {}
}
