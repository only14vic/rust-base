use {
    crate::{
        alloc::{ffi::CString, string::ToString},
        prelude::*
    },
    alloc::{boxed::Box, format, string::String},
    core::{
        ffi::{CStr, c_char},
        mem::{transmute, zeroed},
        ops::{Deref, DerefMut},
        ptr::null_mut,
        str::FromStr,
        sync::atomic::{AtomicBool, Ordering}
    },
    log::{Level, LevelFilter, Log},
    yansi::Paint
};
#[cfg(not(feature = "std"))]
use libc_print::std_name::*;

/// Logging levels for C
#[repr(C)]
#[allow(unused)]
#[allow(clippy::upper_case_acronyms)]
enum LogLevel {
    OFF = 0,   // unreachable!
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

impl Into<LevelFilter> for LogLevel {
    fn into(self) -> LevelFilter {
        unsafe { transmute(self as usize) }
    }
}

/// Initializes logging
#[unsafe(no_mangle)]
pub extern "C" fn log_init() -> &'static mut Logger {
    static INIT: AtomicBool = AtomicBool::new(false);

    if INIT.swap(true, Ordering::SeqCst) == false {
        log::set_logger(Logger::from_static()).unwrap();
    }

    unsafe { Logger::from_static_mut() }
}

/// Logs messages in C
#[unsafe(no_mangle)]
extern "C" fn log_msg(level: LogLevel, target: *const c_char, msg: *const c_char) {
    let target = if target.is_null() {
        module_path!().into()
    } else {
        unsafe { CStr::from_ptr(target.cast()).to_string_lossy() }
    };
    let msg = unsafe { CStr::from_ptr(msg.cast()).to_string_lossy() };
    log::log!(target: &target, level.into(), "{msg}");
}

/// Set max log level in C
#[unsafe(no_mangle)]
extern "C" fn log_max_level(level: LogLevel) {
    log::set_max_level(level.into());
}

/// Logger
#[derive(FromStatic)]
pub struct Logger {
    config: LogConfig,
    file: Option<Box<libc::FILE>>,
    lock: AtomicBool
}

impl Default for Logger {
    fn default() -> Self {
        let mut log = Logger {
            config: Default::default(),
            file: Default::default(),
            lock: Default::default()
        };

        let mut config = LogConfig::default();
        config.load_env();
        log.configure(&config).unwrap();

        log
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.log_close();
    }
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
    pub fn configure(&mut self, config: &LogConfig) -> Void {
        self.log_close();
        self.config.clone_from(config);

        if let Some(path) = self.config.file.as_ref()
            && path.is_empty() == false
        {
            Dirs::mkdir(Dirs::dirname(path))?;
            unsafe {
                let file =
                    libc::fopen(CString::new(path.as_str())?.as_ptr(), c"a+".as_ptr());
                if file.is_null() {
                    Err(format!("Could not open log file: {path}"))?;
                }
                self.file = Box::from_raw(file).into();
            }
        }

        log::set_max_level(self.config.level);

        Env::is_debug().then(|| log::trace!("Configured {:?}", self.config));

        ok()
    }

    /// Close log file descriptor
    #[unsafe(no_mangle)]
    pub extern "C" fn log_close(&mut self) {
        if let Some(file) = self.file.take() {
            unsafe { libc::fclose(Box::into_raw(file)) };

            Env::is_debug().then(|| {
                log::trace!(
                    "LOG FILE CLOSED: {}",
                    self.config
                        .file
                        .as_ref()
                        .unwrap_or(&String::with_capacity(0))
                )
            });
        }
    }

    pub fn get_closer(&'static mut self) -> LogCloser {
        LogCloser { logger: self }
    }

    fn time() -> String {
        unsafe {
            let mut time: libc::timeval = zeroed();
            libc::gettimeofday(&mut time as *mut _, null_mut());
            let local = &*libc::localtime(&time.tv_sec);
            const BUFF_LEN: usize = 60;
            let buff = &mut [0i8; BUFF_LEN] as *mut _;
            #[cfg(target_arch = "aarch64")]
            let buff = buff as *mut u8;

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
            let mut allow = true;

            for value in filter.iter() {
                if let Some(value) = value.strip_prefix("!") {
                    if target.starts_with(value) {
                        allow = false;
                        break;
                    }
                } else if target.starts_with(value) {
                    allow = true;
                    break;
                } else {
                    allow = false;
                }
            }

            if allow == false {
                return;
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

pub struct LogCloser {
    logger: &'static mut Logger
}

impl Drop for LogCloser {
    fn drop(&mut self) {
        self.logger.log_close();
    }
}
