
/// Internal use only. Workaround for unsafe block in log macro.
pub fn __strdup_safe(
    pool: *mut switch_memory_pool_t,
    todup: &str,
    file: *const c_char,
    line: c_int,
) -> *mut c_char {
    unsafe {
        let todup = std::ffi::CString::new(todup).expect("CString::new");
        switch_core_perform_strdup(pool, todup.as_ptr(), file, std::ptr::null(), line)
    }
}

#[macro_export]
macro_rules! strdup {
    ($pool:expr, $todup:expr) => {
        __strdup_safe(
            $pool,
            $todup,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int,
        )
    };
}

/// Internal use only. Workaround for unsafe block in log macro.
pub fn __switch_alloc(
    pool: *mut switch_memory_pool_t,
    size: usize,
    file: *const c_char,
    line: c_int,
) -> *mut c_void {
    unsafe {
        switch_core_perform_alloc(pool, size, file, std::ptr::null(), line)
    }
}

#[macro_export]
macro_rules! switch_alloc {
    ($pool:expr, $size:expr) => {
        __switch_alloc(
            $pool,
            $size,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int,
        )
    };
}
