use {core::panic::PanicInfo, libc::abort, libc_alloc::LibcAlloc};
#[allow(unused_imports)]
use libc_print::std_name::*;

#[global_allocator]
static GLOBAL_ALLOC: LibcAlloc = LibcAlloc;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("ERROR: {info}");
    unsafe { abort() };
}

#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn _Unwind_Resume() {}
