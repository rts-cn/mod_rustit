/// Internal use only. Workaround for unsafe block in log macro.
pub fn __log_printf_safe(
    channel: switch_text_channel_t,
    file: *const c_char,
    line: c_int,
    level: switch_log_level_t,
    s: *const u8,
) {
    unsafe {
        let fmt = CString::new("%s\n").expect("CString::new");
        switch_log_printf(
            channel,
            file,
            std::ptr::null(),
            line,
            std::ptr::null(),
            level,
            fmt.as_ptr(),
            s,
        );
    }
}

#[macro_export]
macro_rules! log_printf {
    ($level:expr, $s:expr) => {
        __log_printf_safe(
            fs::switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int,
            $level,
            $s,
        );
    };
}

/// Calls FreeSWITCH log_printf, but uses Rust format! instead of printf.
#[macro_export]
macro_rules! notice {
    ($s:expr) => (
        let s = concat!($s, "\0");
        __log_printf_safe(
            switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, switch_log_level_t::SWITCH_LOG_NOTICE, s.as_ptr());
    );
    ($fmt:expr, $($arg:expr),*) => (
        let s = format!(concat!($fmt, "\0"), $($arg), *);
        __log_printf_safe(
            switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, switch_log_level_t::SWITCH_LOG_NOTICE, s.as_ptr());
    );
}

#[macro_export]
macro_rules! debug {
    ($s:expr) => (
        let s = concat!($s, "\0");
        __log_printf_safe(
            switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, switch_log_level_t::SWITCH_LOG_DEBUG, s.as_ptr());
    );
    ($fmt:expr, $($arg:expr),*) => (
        let s = format!(concat!($fmt, "\0"), $($arg), *);
        __log_printf_safe(
            switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, switch_log_level_t::SWITCH_LOG_DEBUG, s.as_ptr());
    );
}

#[macro_export]
macro_rules! info {
    ($s:expr) => (
        let s = concat!($s, "\0");
        __log_printf_safe(
            switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, switch_log_level_t::SWITCH_LOG_INFO, s.as_ptr());
    );
    ($fmt:expr, $($arg:expr),*) => (
        let s = format!(concat!($fmt, "\0"), $($arg), *);
        __log_printf_safe(
            switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, switch_log_level_t::SWITCH_LOG_INFO, s.as_ptr());
    );
}

#[macro_export]
macro_rules! error {
    ($s:expr) => (
        let s = concat!($s, "\0");
        __log_printf_safe(
            switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, switch_log_level_t::SWITCH_LOG_ERROR, s.as_ptr());
    );
    ($fmt:expr, $($arg:expr),*) => (
        let s = format!(concat!($fmt, "\0"), $($arg), *);
        __log_printf_safe(
            switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, switch_log_level_t::SWITCH_LOG_ERROR, s.as_ptr());
    );
}

#[macro_export]
macro_rules! warn {
    ($s:expr) => (
        let s = concat!($s, "\0");
        __log_printf_safe(
            switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, switch_log_level_t::SWITCH_LOG_WARNING, s.as_ptr());
    );
    ($fmt:expr, $($arg:expr),*) => (
        let s = format!(concat!($fmt, "\0"), $($arg), *);
        __log_printf_safe(
            switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, switch_log_level_t::SWITCH_LOG_WARNING, s.as_ptr());
    );
}
