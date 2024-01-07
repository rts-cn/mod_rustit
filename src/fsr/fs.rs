#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![allow(improper_ctypes)]

extern crate libc;
use libc::c_char;
use libc::c_int;

std::include!("bindings.rs");

/// This function copies a &str to a C-style nul-terminated char*.
/// It uses malloc, so that other code (FreeSWITCH) can call free() on it.
/// For example, event_header.name is a *mut c_char that FS will free when finished.
pub fn str_to_ptr(s: &str) -> *mut c_char {
    unsafe {
        let res = ::libc::malloc(s.len() + 1) as *mut c_char;
        std::ptr::copy_nonoverlapping(s.as_ptr(), res as *mut u8, s.len());
        *res.offset(s.len() as isize) = 0;
        res
    }
}

/// Take a char*. On null, return None. Otherwise a &str or String.
/// Lossy conversion is applied, so non-UTF-8 char* will result in an allocation
/// and replacement of invalid UTF-8 sequences.
pub unsafe fn ptr_to_str<'a>(p: *const c_char) -> Option<std::borrow::Cow<'a, str>> {
    if p.is_null() {
        return None;
    }
    let cs = std::ffi::CStr::from_ptr(p);
    Some(cs.to_string_lossy())
}

/// Creates a constant nul-terminated *const c_char
#[macro_export]
macro_rules! char_const {
    ($s:expr) => {
        concat!($s, "\n\0").as_ptr() as *const ::libc::c_char
    };
}

/// Internal use only. Workaround for unsafe block in fslog macro.
pub fn __log_printf_safe(
    channel: self::switch_text_channel_t,
    file: *const c_char,
    line: c_int,
    level: self::switch_log_level_t,
    s: *const u8,
) {
    unsafe {
        self::switch_log_printf(
            channel,
            file,
            std::ptr::null(),
            line,
            std::ptr::null(),
            level,
            char_const!("%s"),
            s,
        );
    }
}

/// Calls FreeSWITCH log_printf, but uses Rust format! instead of printf.
/// Be sure to have libc in your Cargo.toml.
#[macro_export]
macro_rules! fslog {
    ($level:expr, $s:expr) => (
        let s = concat!($s, "\0");
        fsr::fs::__log_printf_safe(
            fsr::fs::switch_text_channel_t_SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const libc::c_char,
            line!() as libc::c_int, $level, s.as_ptr());
    );
    ($level:expr, $fmt:expr, $($arg:expr),*) => (
        let s = format!(concat!($fmt, "\0"), $($arg), *);
        fsr::fs::__log_printf_safe(
            fsr::fs::switch_text_channel_t_SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const libc::c_char,
            line!() as libc::c_int, $level, s.as_ptr());
    );
}
