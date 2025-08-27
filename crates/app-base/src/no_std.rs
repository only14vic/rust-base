use {
    alloc::boxed::Box,
    core::{
        panic::PanicInfo,
        ptr::null_mut,
        sync::atomic::{AtomicPtr, Ordering}
    },
    libc::abort,
    libc_alloc::LibcAlloc
};
#[allow(unused_imports)]
use libc_print::std_name::*;

#[global_allocator]
static GLOBAL_ALLOC: LibcAlloc = LibcAlloc;

static PANIC_HANDLER: AtomicPtr<fn(&PanicInfo<'_>)> = AtomicPtr::new(null_mut());

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    let handler = PANIC_HANDLER.load(Ordering::SeqCst);
    if handler.is_null() == false {
        unsafe { (*handler)(info) };
    } else {
        eprintln!("PANIC: {info}");
    }
    unsafe { abort() };
}

pub fn set_panic_handler(handler: Box<fn(&PanicInfo<'_>)>) {
    PANIC_HANDLER.store(Box::leak(handler), Ordering::SeqCst);
}

#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn _Unwind_Resume() {}
