#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![allow(improper_ctypes)]

use std::assert;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;

include!("fs.rs");

pub fn to_string<'a>(p: *const c_char) -> String {
    if p.is_null() {
        return String::from("");
    }
    let cstr = unsafe { std::ffi::CStr::from_ptr(p) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

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
pub struct Event(*mut switch_event_t);
impl Event {
    pub unsafe fn from_ptr(p: *mut switch_event_t) -> Event {
        assert!(!p.is_null());
        Event(p)
    }
    pub fn as_ptr(&self) -> *const switch_event_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut switch_event_t {
        self.0
    }
    pub unsafe fn as_ref(&self) -> &switch_event_t {
        &*self.0
    }
    pub unsafe fn as_mut_ref(&mut self) -> &mut switch_event_t {
        &mut *self.0
    }
    pub fn event_id(&self) -> u32 {
        unsafe { (*self.0).event_id as u32 }
    }
    pub fn priority(&self) -> u32 {
        unsafe { (*self.0).priority as u32 }
    }
    pub fn owner(&self) -> String {
        unsafe { self::to_string((*self.0).owner) }
    }
    pub fn subclass_name(&self) -> String {
        unsafe { self::to_string((*self.0).subclass_name) }
    }
    pub fn body(&self) -> String {
        unsafe { self::to_string((*self.0).body) }
    }
    pub fn key(&self) -> u64 {
        unsafe { (*self.0).key as u64 }
    }
    pub fn flags(&self) -> i32 {
        unsafe { (*self.0).flags }
    }
    pub fn header<'a>(&'a self, name: &str) -> String {
        unsafe {
            let hname: CString = CString::new(name).expect("CString::new");
            let v = switch_event_get_header_idx(self.0, hname.as_ptr(), -1);
            self::to_string(v)
        }
    }
    pub fn headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        unsafe {
            let mut hp = { *self.0 }.headers;
            loop {
                if hp.is_null() {
                    break;
                }
                headers.insert(to_string((*hp).name), to_string((*hp).value));
                hp = (*hp).next;
            }
        }
        headers
    }
    pub fn string<'a>(&'a self) -> String {
        unsafe {
            let mut s: *mut c_char = std::ptr::null_mut();
            switch_event_serialize(
                self.0,
                std::ptr::addr_of_mut!(s),
                switch_bool_t::SWITCH_FALSE,
            );
            let text = self::to_string(s);
            libc::free(s as *mut c_void);
            text
        }
    }
    pub fn json<'a>(&'a self) -> String {
        unsafe {
            let mut s: *mut c_char = std::ptr::null_mut();
            switch_event_serialize_json(self.0, std::ptr::addr_of_mut!(s));
            let text = self::to_string(s);
            libc::free(s as *mut c_void);
            text
        }
    }
}

pub fn event_bind<F>(
    m: &Module,
    id: &str,
    event: switch_event_types_t,
    subclass_name: Option<&str>,
    callback: F,
) -> u64
where
    F: Fn(Event),
{
    unsafe extern "C" fn wrap_callback<F>(e: *mut switch_event_t)
    where
        F: Fn(Event),
    {
        assert!(!e.is_null());
        assert!(!((*e).bind_user_data.is_null()));
        let f = (*e).bind_user_data as *const F;
        let e = Event::from_ptr(e);
        (*f)(e);
    }
    let fp = std::ptr::addr_of!(callback);
    unsafe {
        let id = strdup!(m.pool(), id);
        let subclass_name = subclass_name.map_or(std::ptr::null(), |x| strdup!(m.pool(), x));
        let mut enode = 0 as *mut u64;
        switch_event_bind_removable(
            id,
            event,
            subclass_name,
            Some(wrap_callback::<F>),
            fp as *mut c_void,
            (&mut enode) as *mut _ as *mut *mut switch_event_node_t,
        );
        enode as u64
    }
}

pub fn event_unbind(id: u64) {
    let mut enode = id as *mut u64;
    unsafe {
        switch_event_unbind((&mut enode) as *mut _ as *mut *mut switch_event_node_t);
    }
}

pub struct Module {
    module: *mut switch_loadable_module_interface_t,
    pool: *mut switch_memory_pool_t,
}

impl Module {
    pub unsafe fn from_ptr(
        module: *mut switch_loadable_module_interface_t,
        pool: *mut switch_memory_pool_t,
    ) -> Module {
        assert!(!pool.is_null());
        assert!(!module.is_null());
        Module { module, pool }
    }
    pub fn as_ptr(&mut self) -> *const switch_loadable_module_interface_t {
        self.module
    }
    pub fn as_mut_ptr(&mut self) -> *mut switch_loadable_module_interface_t {
        self.module
    }
    pub unsafe fn as_ref(&self) -> &switch_loadable_module_interface_t {
        &*self.module
    }
    pub unsafe fn as_mut_ref(&self) -> &mut switch_loadable_module_interface_t {
        &mut *self.module
    }

    pub unsafe fn pool(&self) -> *mut switch_memory_pool_t {
        self.pool
    }

    unsafe fn create_int(&self, iname: switch_module_interface_name_t) -> *mut c_void {
        switch_loadable_module_create_interface((*self).module, iname)
    }

    pub fn add_api(&self, name: &str, desc: &str, syntax: &str, func: switch_api_function_t) {
        unsafe {
            let name = strdup!(self.pool(), name);
            let desc = strdup!(self.pool(), desc);
            let syntax = strdup!(self.pool(), syntax);
            let api = self.create_int(switch_module_interface_name_t::SWITCH_API_INTERFACE)
                as *mut switch_api_interface_t;

            assert!(!api.is_null());
            (*api).interface_name = name;
            (*api).desc = desc;
            (*api).syntax = syntax;
            (*api).function = func;
        }
    }

    pub fn add_application(
        &self,
        name: &str,
        short_desc: &str,
        long_desc: &str,
        syntax: &str,
        func: switch_application_function_t,
        flags: switch_application_flag_enum_t,
    ) {
        unsafe {
            let name = strdup!(self.pool(), name);
            let long_desc = strdup!(self.pool(), long_desc);
            let short_desc = strdup!(self.pool(), short_desc);
            let syntax = strdup!(self.pool(), syntax);
            let app = self.create_int(switch_module_interface_name_t::SWITCH_APPLICATION_INTERFACE)
                as *mut switch_application_interface;
            assert!(!app.is_null());
            (*app).interface_name = name;
            (*app).long_desc = long_desc;
            (*app).short_desc = short_desc;
            (*app).syntax = syntax;
            (*app).flags = flags as u32;
            (*app).application_function = func;
        }
    }
}

pub struct Session(*mut switch_core_session_t);
impl Session {
    pub unsafe fn from_ptr(p: *mut switch_core_session_t) -> Session {
        Session(p)
    }
    pub fn as_ptr(&self) -> *const switch_core_session_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut switch_core_session_t {
        self.0
    }
}

pub struct Stream(*mut switch_stream_handle_t);
impl Stream {
    pub unsafe fn from_ptr(p: *mut switch_stream_handle_t) -> Stream {
        assert!(!p.is_null());
        Stream(p)
    }
    pub fn as_ptr(&self) -> *const switch_stream_handle_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut switch_stream_handle_t {
        self.0
    }
    pub fn write(&self, s: &str) {
        let ok = CString::new(s).expect("CString::new failed");
        unsafe {
            (*self.0).write_function.unwrap()(self.0, ok.as_ptr());
        }
    }
}

/// Create FreeSWITCH module interface
///
/// # Examples
/// ```
/// use fsr::switch_status_t;
/// fn examples_mod_load(m: &fsr::Module) -> switch_status_t { SWITCH_STATUS_SUCCESS }
/// fn examples_mod_runtime() -> switch_status_t { SWITCH_STATUS_SUCCESS }
/// fn examples_mod_shutdown() -> switch_status_t { SWITCH_STATUS_SUCCESS }
/// fsr_mod!("mod_examples", examples_mod_load, examples_mod_runtime, examples_mod_shutdown);
///
/// ```
#[macro_export]
macro_rules! fsr_mod {
    ($name:expr,$load:expr,$runtime:expr,$shutdown:expr) => {
        paste::paste! {
        #[no_mangle]
        pub unsafe extern "C" fn _mod_load(
            mod_int: *mut *mut switch_loadable_module_interface,
            mem_pool: *mut switch_memory_pool_t,
        ) -> switch_status_t {
            let name = strdup!(mem_pool, $name);
            *mod_int = switch_loadable_module_create_module_interface(mem_pool, name);
            if (*mod_int).is_null() {
                return switch_status_t::SWITCH_STATUS_MEMERR;
            }
            let mi = &Module::from_ptr(*mod_int, mem_pool);
            $load(mi)
        }

        #[no_mangle]
        pub extern "C" fn _mod_runtime() -> switch_status_t {
            $runtime()
        }

        #[no_mangle]
        pub extern "C" fn _mod_shutdown() -> switch_status_t {
            $shutdown()
        }

        #[no_mangle]
        #[allow(non_upper_case_globals)]
        pub static mut [<$name _module_interface>]: switch_loadable_module_function_table =
            switch_loadable_module_function_table {
                switch_api_version: SWITCH_API_VERSION as i32,
                load: Some(_mod_load),
                shutdown: Some(_mod_shutdown),
                runtime: Some(_mod_runtime),
                flags: switch_module_flag_enum_t::SMODF_NONE as u32,
            };
        }
    };
}

/// Add FreeSWITCH Appliction
///
/// This macro will add a FreeSWICH application
///
/// # Examples
///
/// ```
/// use fsr::switch_application_flag_enum_t;
/// fn examples(session &fsr::session, cmd String)
/// {
///     info!({}, cmd);
/// }
/// fsr_app!(module_interface, "app_name", "long_desc", "short_desc", "syntax", examples, SAF_NONE);
/// ```
#[macro_export]
macro_rules! fsr_app {
    ($module:expr,$name:expr,$short_desc:expr,$long_desc:expr,$syntax:expr,$callback:ident, $flag:expr) => {
        paste::paste! {
            unsafe extern "C" fn [<_app_wrap_ $callback>](
                session: *mut switch_core_session_t,
                cmd: *const ::std::os::raw::c_char
            ) {
                let session = &fsr::Session::from_ptr(session);
                $callback(session, to_string(cmd));
            }

            $module.add_application($name, $short_desc, $long_desc, $syntax, Some([<_app_wrap_ $callback>]), $flag);
        }
    };
}

/// Add FreeSWITCH API
///
/// This macro will add a FreeSWICH API
/// # Examples
///
/// ```
/// fn examples(session &fsr::Session, cmd String, stream &fsr::Stream) -> fsr::switch_status_t
/// {
///    info!({}, cmd);
//     stream.write("OK");
///    switch_status_t::SWITCH_STATUS_SUCCESS
/// }
/// fsr_api!(module_interface, "api_name", "desc", "syntax", examples);
/// ```
#[macro_export]
macro_rules! fsr_api {
    ($module:expr,$name:expr,$desc:expr,$syntax:expr,$callback:ident) => {
        paste::paste! {
                unsafe extern "C" fn [<_api_wrap_ $callback>](
                    cmd: *const std::os::raw::c_char,
                    session: *mut fsr::switch_core_session,
                    stream: *mut fsr::switch_stream_handle_t,
                ) -> switch_status_t {
                    let session = &fsr::Session::from_ptr(session);
                    let stream = &fsr::Stream::from_ptr(stream);
                    $callback(session, to_string(cmd), stream)
                }
            $module.add_api($name, $desc, $syntax, Some([<_api_wrap_ $callback>]));
        }
    };
}

pub fn get_variable(s: &str) -> String {
    let tmp_str = CString::new(s).unwrap();
    let val = unsafe { switch_core_get_variable(tmp_str.as_ptr()) };
    to_string(val)
}

pub fn check_acl(ip: &str, list: &str) -> bool {
    let cstr_ip = CString::new(ip).unwrap();
    let cstr_list = CString::new(list).unwrap();
    let mut token = std::ptr::null() as *const c_char;

    let r = unsafe {
        switch_check_network_list_ip_token(
            cstr_ip.as_ptr(),
            cstr_list.as_ptr(),
            std::ptr::addr_of_mut!(token),
        )
    };
    if r == switch_bool_t::SWITCH_TRUE {
        true
    } else {
        false
    }
}
pub fn api_exec(cmd: &str, arg: &str) -> Result<String, String> {
    unsafe {
        let data_size: usize = 1024;
        let data = libc::malloc(data_size);
        libc::memset(data, 0, data_size);
        let stream = &mut switch_stream_handle_t {
            read_function: None,
            write_function: Some(switch_console_stream_write),
            raw_write_function: Some(switch_console_stream_raw_write),
            data,
            data_size,
            data_len: 0,
            alloc_len: data_size,
            alloc_chunk: data_size,
            param_event: std::ptr::null_mut() as *mut switch_event,
            end: data,
        };

        let api_cmd = CString::new(cmd).unwrap();
        let api_arg = CString::new(arg).unwrap();

        let status = switch_api_execute(
            api_cmd.as_ptr(),
            api_arg.as_ptr(),
            std::ptr::null_mut() as *mut switch_core_session_t,
            stream as *mut switch_stream_handle_t,
        );

        let ret: String = to_string((*stream).data as *const c_char);
        libc::free((*stream).data);
        if status == switch_status_t::SWITCH_STATUS_SUCCESS {
            Ok(ret)
        } else {
            Err(format!("-ERR %s Command not found!{}\n", cmd))
        }
    }
}
