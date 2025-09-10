use {
    super::{AppConstomConfig, AppEvent},
    crate::prelude::*,
    alloc::{boxed::Box, vec::Vec},
    core::{
        ffi::{CStr, c_char, c_int, c_uint},
        mem::transmute
    }
};

pub type App = super::App<AppConstomConfig>;

pub type AppModuleC = extern "C" fn(*mut App, AppEvent) -> c_uint;

#[unsafe(no_mangle)]
unsafe extern "C" fn app_new(modules: *mut AppModuleC, count: c_uint) -> *mut App {
    let modules = unsafe { Vec::from_raw_parts(modules, count as usize, count as usize) };
    let app = App::new(modules.into_iter().map(|m| unsafe { transmute(m) }));
    Box::into_raw(app.into())
}

#[unsafe(no_mangle)]
#[allow(unused_variables)]
unsafe extern "C" fn app_boot(app: *mut App, argc: c_int, argv: *const *const c_char) {
    Di::from_static().set(unsafe { Box::from_raw(app) });

    let app = unsafe { &mut *app };

    #[cfg(feature = "std")]
    let _ = app.boot().unwrap_or_else(|e| panic!("{e}"));

    #[cfg(not(feature = "std"))]
    let _ = app.boot(argc, argv).unwrap_or_else(|e| panic!("{e}"));
}

#[unsafe(no_mangle)]
unsafe extern "C" fn app_run(app: *mut App) {
    let app = unsafe { &mut *app };
    app.run().unwrap_or_else(|e| panic!("{e}"));
}

#[unsafe(no_mangle)]
unsafe extern "C" fn app_free(app: *mut App) {
    let _ = unsafe { Box::from_raw(app) };
}

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
unsafe extern "C" fn app_register_command(
    app: *mut App,
    command: *const c_char,
    module: AppModuleC
) {
    unsafe {
        let app = &mut *app;
        let command = CStr::from_ptr(command).to_str().unwrap();
        app.register_command(command, transmute(module));
    }
}
