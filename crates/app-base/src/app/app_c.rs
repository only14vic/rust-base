use {
    super::{AppEvent, AppSimpleConfig},
    crate::prelude::*,
    alloc::boxed::Box,
    core::{
        error::Error,
        ffi::{CStr, c_char, c_int, c_uint, c_void},
        mem::transmute,
        slice::from_raw_parts
    }
};

pub type App = super::App<AppSimpleConfig>;

pub type AppModuleC = extern "C" fn(*mut App, AppEvent) -> *const c_void;

#[unsafe(no_mangle)]
unsafe extern "C" fn app_new(modules: *mut AppModuleC, count: c_uint) -> *mut App {
    let modules = unsafe { from_raw_parts(modules, count as usize) };
    let modules = modules.iter().map(|m| unsafe { transmute(*m) });
    let app = App::new(modules);
    Box::into_raw(app.into())
}

#[unsafe(no_mangle)]
#[allow(unused_variables)]
unsafe extern "C" fn app_boot(app: *mut App, argc: c_int, argv: *const *const c_char) {
    Di::from_static().add(unsafe { Box::from_raw(app) });

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
unsafe extern "C" fn app_register_command(
    app: *mut App,
    command: *const c_char,
    module: AppModuleC
) {
    unsafe {
        let app = &mut *app;
        let command = CStr::from_ptr(command).to_str().unwrap();
        #[allow(clippy::missing_transmute_annotations)]
        app.register_command(command, transmute(module));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn app_unregister_command(app: *mut App, command: *const c_char) {
    unsafe {
        let app = &mut *app;
        let command = CStr::from_ptr(command).to_str().unwrap();
        app.unregister_command(command);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn app_register_module(app: *mut App, module: AppModuleC) {
    unsafe {
        let app = &mut *app;
        #[allow(clippy::missing_transmute_annotations)]
        app.register_module(transmute(module));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn app_unregister_module(app: *mut App, module: AppModuleC) {
    unsafe {
        let app = &mut *app;
        #[allow(clippy::missing_transmute_annotations)]
        app.unregister_module(transmute(module));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn app_error(err: *const c_char) -> *const c_void {
    unsafe {
        let err: Box<dyn Error> = CStr::from_ptr(err).to_string_lossy().into();
        Box::into_raw(err).cast()
    }
}
