pub fn mem_stats() {
    #[cfg(not(target_env = "musl"))]
    unsafe {
        libc::malloc_stats()
    };

    #[cfg(any(feature = "std", target_env = "musl"))]
    if let Some(usage) = memory_stats::memory_stats() {
        eprintln!("Phys use bytes = {}", usage.physical_mem);
        eprintln!("Virt use bytes = {}", usage.virtual_mem);
    }
}
