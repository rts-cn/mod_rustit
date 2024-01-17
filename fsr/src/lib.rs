#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![allow(improper_ctypes)]

use serde::{Deserialize, Serialize};
use std::assert;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;

pub mod fs;

pub fn to_string<'a>(p: *const c_char) -> String {
    if p.is_null() {
        return String::from("");
    }
    let cstr = unsafe { std::ffi::CStr::from_ptr(p) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

/// Internal use only. Workaround for unsafe block in fslog macro.
pub fn __log_printf_safe(
    channel: fs::switch_text_channel_t,
    file: *const c_char,
    line: c_int,
    level: fs::switch_log_level_t,
    s: *const u8,
) {
    unsafe {
        let fmt = CString::new("%s\n").expect("CString::new");
        fs::switch_log_printf(
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

/// Calls FreeSWITCH log_printf, but uses Rust format! instead of printf.
#[macro_export]
macro_rules! fslog {
    ($level:expr, $s:expr) => (
        let s = concat!($s, "\0");
        __log_printf_safe(
            fs::switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, $level, s.as_ptr());
    );
    ($level:expr, $fmt:expr, $($arg:expr),*) => (
        let s = format!(concat!($fmt, "\0"), $($arg), *);
        __log_printf_safe(
            fs::switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int, $level, s.as_ptr());
    );
}

/// Internal use only. Workaround for unsafe block in fslog macro.
pub fn __strdup_safe(
    pool: *mut fs::switch_memory_pool_t,
    todup: &str,
    file: *const c_char,
    line: c_int,
) -> *mut c_char {
    unsafe {
        let todup = std::ffi::CString::new(todup).expect("CString::new");
        fs::switch_core_perform_strdup(pool, todup.as_ptr(), file, std::ptr::null(), line)
    }
}

#[macro_export]
macro_rules! fs_strdup {
    ($pool:expr, $todup:expr) => {
        __strdup_safe(
            $pool,
            $todup,
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            line!() as std::os::raw::c_int,
        )
    };
}

pub struct Session(*mut fs::switch_core_session_t);
impl Session {
    pub unsafe fn from_ptr(p: *mut fs::switch_core_session_t) -> Session {
        assert!(!p.is_null());
        Session(p)
    }
    pub fn as_ptr(&self) -> *const fs::switch_core_session_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut fs::switch_core_session_t {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    event_id: u32,
    priority: u32,
    owner: String,
    subclass_name: String,
    body: String,
    key: u64,
    flags: i32,
    headers: HashMap<String, String>,
}

impl Event {
    pub unsafe fn from_raw(p: *mut fs::switch_event_t) -> Event {
        assert!(!p.is_null());
        let mut headers = HashMap::new();
        let mut hp = (*p).headers;
        loop {
            if hp.is_null() {
                break;
            }
            headers.insert(to_string((*hp).name), to_string((*hp).value));
            hp = (*hp).next;
        }
        Event {
            event_id: (*p).event_id as u32,
            priority: (*p).priority as u32,
            owner: to_string((*p).owner),
            subclass_name: to_string((*p).subclass_name),
            body: to_string((*p).body),
            key: (*p).key,
            flags: (*p).flags,
            headers,
        }
    }
    pub fn event_id(&self) -> u32 {
        self.event_id
    }
    pub fn priority(&self) -> u32 {
        self.priority
    }
    pub fn owner(&self) -> &String {
        &self.owner
    }
    pub fn subclass_name(&self) -> &String {
        &self.subclass_name
    }
    pub fn body(&self) -> &String {
        &self.body
    }
    pub fn key(&self) -> u64 {
        self.key as u64
    }
    pub fn flags(&self) -> i32 {
        self.flags as i32
    }
    pub fn header(&self, name: &String) -> Option<&String> {
        self.headers.get(name)
    }
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

pub fn event_bind<F>(
    mi: &ModInterface,
    id: &str,
    event: fs::switch_event_types_t,
    subclass_name: Option<&str>,
    callback: F,
) -> u64
where
    F: Fn(Event),
{
    unsafe extern "C" fn wrap_callback<F>(e: *mut fs::switch_event_t)
    where
        F: Fn(Event),
    {
        assert!(!e.is_null());
        assert!(!((*e).bind_user_data.is_null()));
        let f = (*e).bind_user_data as *const F;
        let e = Event::from_raw(e);
        (*f)(e);
    }
    let fp = std::ptr::addr_of!(callback);
    unsafe {
        let id = fs_strdup!(mi.pool(), id);
        let subclass_name = subclass_name.map_or(std::ptr::null(), |x| fs_strdup!(mi.pool(), x));
        let mut enode = 0 as *mut u64;
        fs::switch_event_bind_removable(
            id,
            event,
            subclass_name,
            Some(wrap_callback::<F>),
            fp as *mut c_void,
            (&mut enode) as *mut _ as *mut *mut fs::switch_event_node_t,
        );
        enode as u64
    }
}

pub fn event_unbind(id: u64) {
    let mut enode = id as *mut u64;
    unsafe {
        fs::switch_event_unbind((&mut enode) as *mut _ as *mut *mut fs::switch_event_node_t);
    }
}

pub struct ModInterface {
    module: *mut fs::switch_loadable_module_interface_t,
    pool: *mut fs::switch_memory_pool_t,
}

impl ModInterface {
    pub unsafe fn from_ptr(
        module: *mut fs::switch_loadable_module_interface_t,
        pool: *mut fs::switch_memory_pool_t,
    ) -> ModInterface {
        assert!(!pool.is_null());
        assert!(!module.is_null());
        ModInterface { module, pool }
    }
    pub fn as_ptr(&mut self) -> *const fs::switch_loadable_module_interface_t {
        self.module
    }
    pub fn as_mut_ptr(&mut self) -> *mut fs::switch_loadable_module_interface_t {
        self.module
    }
    pub unsafe fn as_ref(&self) -> &fs::switch_loadable_module_interface_t {
        &*self.module
    }
    pub unsafe fn as_mut_ref(&self) -> &mut fs::switch_loadable_module_interface_t {
        &mut *self.module
    }

    pub unsafe fn pool(&self) -> *mut fs::switch_memory_pool_t {
        self.pool
    }

    unsafe fn create_int(&self, iname: fs::switch_module_interface_name_t) -> *mut c_void {
        fs::switch_loadable_module_create_interface((*self).module, iname)
    }

    pub fn add_api(&self, name: &str, desc: &str, syntax: &str, func: fs::switch_api_function_t) {
        unsafe {
            let name = fs_strdup!(self.pool(), name);
            let desc = fs_strdup!(self.pool(), desc);
            let syntax = fs_strdup!(self.pool(), syntax);
            let api = self.create_int(fs::switch_module_interface_name_t::SWITCH_API_INTERFACE)
                as *mut fs::switch_api_interface_t;

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
        long_desc: &str,
        short_desc: &str,
        syntax: &str,
        func: fs::switch_application_function_t,
        flags: fs::switch_application_flag_enum_t,
    ) {
        unsafe {
            let name = fs_strdup!(self.pool(), name);
            let long_desc = fs_strdup!(self.pool(), long_desc);
            let short_desc = fs_strdup!(self.pool(), short_desc);
            let syntax = fs_strdup!(self.pool(), syntax);
            let app = self
                .create_int(fs::switch_module_interface_name_t::SWITCH_APPLICATION_INTERFACE)
                as *mut fs::switch_application_interface;
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

#[macro_export]
macro_rules! fsr_export_mod {
    ($table:ident,$name:expr,$load:expr,$runtime:expr,$shutdown:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn _mod_load(
            mod_int: *mut *mut fs::switch_loadable_module_interface,
            mem_pool: *mut fs::switch_memory_pool_t,
        ) -> fs::switch_status_t {
            let name = CString::new($name).expect("CString::new failed");
            let name = fs::switch_core_perform_strdup(
                mem_pool,
                name.as_ptr(),
                concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
                std::ptr::null(),
                line!() as std::os::raw::c_int,
            );
            *mod_int = fs::switch_loadable_module_create_module_interface(mem_pool, name);
            if (*mod_int).is_null() {
                return fs::switch_status_t::SWITCH_STATUS_MEMERR;
            }
            let mi = &ModInterface::from_ptr(*mod_int, mem_pool);
            $load(mi)
        }

        #[no_mangle]
        pub extern "C" fn _mod_runtime() -> fs::switch_status_t {
            $runtime()
        }

        #[no_mangle]
        pub extern "C" fn _mod_shutdown() -> fs::switch_status_t {
            $shutdown()
        }

        #[no_mangle]
        #[allow(non_upper_case_globals)]
        pub static mut $table: fs::switch_loadable_module_function_table =
            fs::switch_loadable_module_function_table {
                switch_api_version: fs::SWITCH_API_VERSION as i32,
                load: Some(_mod_load),
                shutdown: Some(_mod_shutdown),
                runtime: Some(_mod_runtime),
                flags: fs::switch_module_flag_enum_t::SMODF_NONE as u32,
            };
    };
}
