pub fn mem_stats() {
    #[cfg(not(target_env = "musl"))]
    unsafe {
        libc::malloc_stats()
    };

    #[cfg(feature = "std")]
    if let Some(usage) = memory_stats::memory_stats() {
        eprintln!("Phys use bytes = {}", usage.physical_mem);
        eprintln!("Virt use bytes = {}", usage.virtual_mem);
    }
}
