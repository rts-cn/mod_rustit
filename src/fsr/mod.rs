#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![allow(improper_ctypes)]

extern crate libc;
use libc::c_char;
use libc::c_void;
use std::borrow::Cow;
use std::ffi::CString;

pub mod fs;

macro_rules! ptr_not_null {
    ($x:expr) => {
        if $x.is_null() {
            panic!(concat!(stringify!($x), "is null."))
        }
    };
}

pub struct CoreSession(*mut fs::switch_core_session_t);
impl CoreSession {
    pub unsafe fn from_ptr(p: *mut fs::switch_core_session_t) -> CoreSession {
        ptr_not_null!(p);
        CoreSession(p)
    }
    pub fn as_ptr(&self) -> *const fs::switch_core_session_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut fs::switch_core_session_t {
        self.0
    }
    // No ref access, since switch_core_session is opaque
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
    pub fn owner(&self) -> Option<Cow<str>> {
        unsafe { fs::ptr_to_str((*self.0).owner) }
    }
    pub fn subclass_name(&self) -> Option<Cow<str>> {
        unsafe { fs::ptr_to_str((*self.0).subclass_name) }
    }
    pub fn body(&self) -> Option<Cow<str>> {
        unsafe { fs::ptr_to_str((*self.0).body) }
    }
    pub fn key(&self) -> u64 {
        unsafe { (*self.0).key as u64 }
    }
    pub fn flags(&self) -> isize {
        unsafe { (*self.0).flags as isize }
    }
    pub fn header<'a>(&'a self, name: &str) -> Option<Cow<'a, str>> {
        unsafe {
            let hname = CString::new(name).unwrap();
            let v = fs::switch_event_get_header_idx(self.0, hname.as_ptr(), -1);
            fs::ptr_to_str(v)
        }
    }
    pub fn string<'a>(&'a self) -> String {
        unsafe {
            let mut s: *mut c_char = std::ptr::null_mut();
            fs::switch_event_serialize(
                self.0,
                std::ptr::addr_of_mut!(s),
                fs::switch_bool_t_SWITCH_FALSE,
            );
            let text = fs::ptr_to_str(s).unwrap_or(Cow::Borrowed("")).to_string();
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
    pub fn name(&self) -> Cow<str> {
        unsafe { fs::ptr_to_str((*self.0).name).expect("event_header.name cannot be null.") }
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
    let id = fs::str_to_ptr(id);
    let subclass_name = subclass_name.map_or(std::ptr::null(), |x| fs::str_to_ptr(x));
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

pub enum Stream {} // Temp until wrap stream
pub type ApiFunc = fn(String, Option<&CoreSession>, Stream);
pub type ApiRawFunc = unsafe extern "C" fn(
    cmd: *const c_char,
    session: *mut fs::switch_core_session_t,
    stream: *mut fs::switch_stream_handle_t,
) -> fs::switch_status_t;
pub type AppRawFunc =
    unsafe extern "C" fn(session: *mut fs::switch_core_session_t, data: *const c_char);

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

    pub fn add_raw_api(&self, name: &str, desc: &str, syntax: &str, func: ApiRawFunc) {
        let name = fs::str_to_ptr(name);
        let desc = fs::str_to_ptr(desc);
        let syntax = fs::str_to_ptr(syntax);
        unsafe {
            let ai = self.create_int(fs::switch_module_interface_name_t_SWITCH_API_INTERFACE)
                as *mut fs::switch_api_interface_t;
            ptr_not_null!(ai);
            (*ai).interface_name = name;
            (*ai).desc = desc;
            (*ai).syntax = syntax;
            (*ai).function = Some(func);
        }
    }

    pub fn add_raw_application(
        &self,
        name: &str,
        long_desc: &str,
        short_desc: &str,
        syntax: &str,
        func: AppRawFunc,
        flags: fs::switch_application_flag_enum_t,
    ) {
        let name = fs::str_to_ptr(name);
        let long_desc = fs::str_to_ptr(long_desc);
        let short_desc = fs::str_to_ptr(short_desc);
        let syntax = fs::str_to_ptr(syntax);
        unsafe {
            let ai = self
                .create_int(fs::switch_module_interface_name_t_SWITCH_APPLICATION_INTERFACE)
                as *mut fs::switch_application_interface;
            ptr_not_null!(ai);
            (*ai).interface_name = name;
            (*ai).long_desc = long_desc;
            (*ai).short_desc = short_desc;
            (*ai).syntax = syntax;
            (*ai).flags = flags as u32;
            (*ai).application_function = Some(func);
        }
    }

    // Doing safe versions is a pain. Macros are ugly. Need to use libffi or similar
    // to dynamically create thunks that'll wrap the safe functions.
    // fn add_api(&mut self, name: &str, desc: &str, syntax: &str, func: ApiFunc) {
    //     self.add_raw_api(name, desc, syntax, TODO_ALLOC_TRAMPOLINE(func));
    // }
}

// Module Loading/Definition
pub struct ModDefinition {
    pub name: &'static str,
    pub load: fn(&ModInterface) -> fs::switch_status_t,
    pub shutdown: Option<fn() -> fs::switch_status_t>,
    pub runtime: Option<fn() -> fs::switch_status_t>,
}

pub unsafe fn wrap_mod_load(
    mod_def: &ModDefinition,
    mod_int: *mut *mut fs::switch_loadable_module_interface_t,
    mem_pool: *mut fs::switch_memory_pool_t,
) -> fs::switch_status_t {
    // Name should be a constant [u8], but we'd need some macro or something
    // to ensure null termination. Leaking the name here shouldn't matter.
    // CString's into_raw pointer is not free()'able fwiw
    let name = CString::new(mod_def.name).unwrap().into_raw();
    *mod_int = fs::switch_loadable_module_create_module_interface(mem_pool, name);
    if (*mod_int).is_null() {
        return fs::switch_status_t_SWITCH_STATUS_MEMERR;
    }
    let mi = &ModInterface::from_ptr(*mod_int);
    (mod_def.load)(mi)
}

pub fn wrap_mod_runtime(mod_def: &ModDefinition) -> fs::switch_status_t {
    if let Some(func) = mod_def.runtime {
        func()
    } else {
        fs::switch_status_t_SWITCH_STATUS_SUCCESS
    }
}

pub fn wrap_mod_shutdown(mod_def: &ModDefinition) -> fs::switch_status_t {
    if let Some(func) = mod_def.shutdown {
        func()
    } else {
        fs::switch_status_t_SWITCH_STATUS_SUCCESS
    }
}

/// This macro needs to be called once in the module. It will generate the definitions
/// required to be loaded by FreeSWITCH. FS requires the exported table to have a name
/// of <filename>_module_interface. If your mod is called mod_foo, then the first param
/// to this macro must be mod_foo_module_interface.
/// The second parameter must be a static (global) ModDefinition.
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
                switch_api_version: 5,
                load: Some(_mod_load),
                shutdown: None,
                runtime: None,
                flags: fs::switch_module_flag_enum_t_SMODF_NONE as u32,
            };
    };
}
