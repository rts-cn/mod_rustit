#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![allow(improper_ctypes)]

extern crate libc;
use libc::c_char;
use libc::c_int;
use libc::c_void;
use std::ffi::CString;

pub mod bindings;
use bindings as fs;

macro_rules! ptr_not_null {
    ($x:expr) => {
        if $x.is_null() {
            panic!(concat!(stringify!($x), "is null."))
        }
    };
}

/// Creates a constant nul-terminated *const c_char
#[macro_export]
macro_rules! char_const {
    ($s:expr) => {
        concat!($s, "\n\0").as_ptr() as *const ::libc::c_char
    };
}

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
pub unsafe fn to_string<'a>(p: *const c_char) -> String {
    if p.is_null() {
        return "".to_string();
    }
    let cs = std::ffi::CStr::from_ptr(p);
    cs.to_string_lossy().to_string()
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
        fs::switch_log_printf(
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
        fsr::__log_printf_safe(
            fs::switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const libc::c_char,
            line!() as libc::c_int, $level, s.as_ptr());
    );
    ($level:expr, $fmt:expr, $($arg:expr),*) => (
        let s = format!(concat!($fmt, "\0"), $($arg), *);
        fsr::__log_printf_safe(
            fs::switch_text_channel_t::SWITCH_CHANNEL_ID_LOG,
            concat!(file!(), '\0').as_ptr() as *const libc::c_char,
            line!() as libc::c_int, $level, s.as_ptr());
    );
}

pub struct Session(*mut fs::switch_core_session_t);
impl Session {
    pub unsafe fn from_ptr(p: *mut fs::switch_core_session_t) -> Session {
        ptr_not_null!(p);
        Session(p)
    }
    pub fn as_ptr(&self) -> *const fs::switch_core_session_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut fs::switch_core_session_t {
        self.0
    }
}

pub struct Event(*mut fs::switch_event_t);
impl Event {
    pub unsafe fn from_ptr(p: *mut fs::switch_event_t) -> Event {
        ptr_not_null!(p);
        Event(p)
    }
    pub fn as_ptr(&self) -> *const fs::switch_event_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut fs::switch_event_t {
        self.0
    }
    pub unsafe fn as_ref(&self) -> &fs::switch_event_t {
        &*self.0
    }
    pub unsafe fn as_mut_ref(&mut self) -> &mut fs::switch_event_t {
        &mut *self.0
    }
    pub fn event_id(&self) -> fs::switch_event_types_t {
        unsafe { (*self.0).event_id }
    }
    pub fn priority(&self) -> fs::switch_priority_t {
        unsafe { (*self.0).priority }
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
    pub fn flags(&self) -> isize {
        unsafe { (*self.0).flags as isize }
    }
    pub fn header<'a>(&'a self, name: &str) -> String {
        unsafe {
            let hname: CString = CString::new(name).unwrap();
            let v = fs::switch_event_get_header_idx(self.0, hname.as_ptr(), -1);
            self::to_string(v)
        }
    }
    pub fn string<'a>(&'a self) -> String {
        unsafe {
            let mut s: *mut c_char = std::ptr::null_mut();
            fs::switch_event_serialize(
                self.0,
                std::ptr::addr_of_mut!(s),
                fs::switch_bool_t::SWITCH_FALSE,
            );
            let text = self::to_string(s);
            libc::free(s as *mut c_void);
            text
        }
    }
}

pub struct EventHeader(*mut fs::switch_event_header_t);
impl EventHeader {
    pub unsafe fn from_ptr(p: *mut fs::switch_event_header_t) -> EventHeader {
        ptr_not_null!(p);
        EventHeader(p)
    }
    pub fn as_ptr(&self) -> *const fs::switch_event_header_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut fs::switch_event_header_t {
        self.0
    }
    pub unsafe fn as_ref(&self) -> &fs::switch_event_header_t {
        &*self.0
    }
    pub unsafe fn as_mut_ref(&mut self) -> &mut fs::switch_event_header_t {
        &mut *self.0
    }
    pub fn name(&self) -> String {
        unsafe { self::to_string((*self.0).name) }
    }
}

pub fn event_bind<F>(
    id: &str,
    event: fs::switch_event_types_t,
    subclass_name: Option<&str>,
    callback: F,
) -> u64
where
    F: Fn(Event),
{
    // TODO: Can you modify events in the callback?
    unsafe extern "C" fn wrap_callback<F>(e: *mut fs::switch_event_t)
    where
        F: Fn(Event),
    {
        ptr_not_null!(e);
        ptr_not_null!((*e).bind_user_data);
        let f = (*e).bind_user_data as *mut F;
        let e = Event::from_ptr(e);
        (*f)(e);
    }

    let bx = std::boxed::Box::new(callback);
    let fp = std::boxed::Box::into_raw(bx);
    let id = self::str_to_ptr(id);
    let subclass_name = subclass_name.map_or(std::ptr::null(), |x| self::str_to_ptr(x));
    unsafe {
        let mut enode = 0 as *mut u64;
        fs::switch_event_bind_removable(
            id,
            event,
            subclass_name,
            Some(wrap_callback::<F>),
            fp as *mut libc::c_void,
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

pub struct ModInterface(*mut fs::switch_loadable_module_interface_t);

impl ModInterface {
    pub unsafe fn from_ptr(p: *mut fs::switch_loadable_module_interface_t) -> ModInterface {
        ptr_not_null!(p);
        ModInterface(&mut *p)
    }
    pub fn as_ptr(&mut self) -> *const fs::switch_loadable_module_interface_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut fs::switch_loadable_module_interface_t {
        self.0
    }
    pub unsafe fn as_ref(&self) -> &fs::switch_loadable_module_interface_t {
        &*self.0
    }
    pub unsafe fn as_mut_ref(&self) -> &mut fs::switch_loadable_module_interface_t {
        &mut *self.0
    }

    unsafe fn create_int(&self, iname: fs::switch_module_interface_name_t) -> *mut c_void {
        fs::switch_loadable_module_create_interface((*self).0, iname)
    }

    pub fn add_api(&self, name: &str, desc: &str, syntax: &str, func: fs::switch_api_function_t) {
        let name = self::str_to_ptr(name);
        let desc = self::str_to_ptr(desc);
        let syntax = self::str_to_ptr(syntax);
        unsafe {
            let api = self.create_int(fs::switch_module_interface_name_t::SWITCH_API_INTERFACE)
                as *mut fs::switch_api_interface_t;
            ptr_not_null!(api);
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
        let name = self::str_to_ptr(name);
        let long_desc = self::str_to_ptr(long_desc);
        let short_desc = self::str_to_ptr(short_desc);
        let syntax = self::str_to_ptr(syntax);
        unsafe {
            let ai = self
                .create_int(fs::switch_module_interface_name_t::SWITCH_APPLICATION_INTERFACE)
                as *mut fs::switch_application_interface;
            ptr_not_null!(ai);
            (*ai).interface_name = name;
            (*ai).long_desc = long_desc;
            (*ai).short_desc = short_desc;
            (*ai).syntax = syntax;
            (*ai).flags = flags as u32;
            (*ai).application_function = func;
        }
    }

    // Doing safe versions is a pain. Macros are ugly. Need to use libffi or similar
    // to dynamically create thunks that'll wrap the safe functions.
    // fn add_api(&mut self, name: &str, desc: &str, syntax: &str, func: ApiFunc) {
    //     self.add_raw_api(name, desc, syntax, TODO_ALLOC_TRAMPOLINE(func));
    // }
}

// Module Loading/Definition
pub struct Module {
    pub name: &'static str,
    pub load: fn(&ModInterface) -> fs::switch_status_t,
    pub shutdown: Option<fn() -> fs::switch_status_t>,
    pub runtime: Option<fn() -> fs::switch_status_t>,
}

pub unsafe fn wrap_mod_load(
    mod_def: &Module,
    mod_int: *mut *mut fs::switch_loadable_module_interface_t,
    mem_pool: *mut fs::switch_memory_pool_t,
) -> fs::switch_status_t {
    // Name should be a constant [u8], but we'd need some macro or something
    // to ensure null termination. Leaking the name here shouldn't matter.
    // CString's into_raw pointer is not free()'able fwiw
    let name = CString::new(mod_def.name).unwrap().into_raw();
    *mod_int = fs::switch_loadable_module_create_module_interface(mem_pool, name);
    if (*mod_int).is_null() {
        return fs::switch_status_t::SWITCH_STATUS_MEMERR;
    }
    let mi = &ModInterface::from_ptr(*mod_int);
    (mod_def.load)(mi)
}

pub fn wrap_mod_runtime(mod_def: &Module) -> fs::switch_status_t {
    if let Some(func) = mod_def.runtime {
        func()
    } else {
        fs::switch_status_t::SWITCH_STATUS_SUCCESS
    }
}

pub fn wrap_mod_shutdown(mod_def: &Module) -> fs::switch_status_t {
    if let Some(func) = mod_def.shutdown {
        func()
    } else {
        fs::switch_status_t::SWITCH_STATUS_SUCCESS
    }
}

/// This macro needs to be called once in the module. It will generate the definitions
/// required to be loaded by FreeSWITCH. FS requires the exported table to have a name
/// of <filename>_module_interface. If your mod is called mod_foo, then the first param
/// to this macro must be mod_foo_module_interface.
/// The second parameter must be a static (global) Module.
#[macro_export]
macro_rules! fsr_export_mod {
    ($table:ident, $def:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn _mod_load(
            mod_int: *mut *mut fs::switch_loadable_module_interface,
            mem_pool: *mut fs::switch_memory_pool_t,
        ) -> fs::switch_status_t {
            if let Some(_) = $def.runtime {
                $table.runtime = Some(_mod_runtime);
            }
            if let Some(_) = $def.shutdown {
                $table.shutdown = Some(_mod_shutdown);
            }
            wrap_mod_load(&$def, mod_int, mem_pool)
        }

        #[no_mangle]
        pub extern "C" fn _mod_runtime() -> fs::switch_status_t {
            wrap_mod_runtime(&$def)
        }

        #[no_mangle]
        pub extern "C" fn _mod_shutdown() -> fs::switch_status_t {
            wrap_mod_shutdown(&$def)
        }

        #[no_mangle]
        #[allow(non_upper_case_globals)]
        pub static mut $table: fs::switch_loadable_module_function_table =
            fs::switch_loadable_module_function_table {
                switch_api_version: fs::SWITCH_API_VERSION as i32,
                load: Some(_mod_load),
                shutdown: None,
                runtime: None,
                flags: fs::switch_module_flag_enum_t::SMODF_NONE as u32,
            };
    };
}
