#[macro_use]
extern crate libc;
use std::{sync::Mutex, thread, time};
use std::ffi::CString;
use lazy_static::lazy_static;

pub mod fsr;
use fsr::*;

struct Global {
    event_id: u64,
}

impl Global {
    fn new() -> Global {
        Global { event_id: 0 }
    }
}

static MODULE_NAME: &str = "mod_zrapi";

lazy_static! {
    static ref GLOBALS: Mutex<Global> = Mutex::new(Global::new());
}

fn heartbeat_binding(e: fsr::Event) {
    let s = e.subclass_name();
    let b = e.body();
    fslog!(
        fs::switch_log_level_t::SWITCH_LOG_INFO,
        "{:?}/{:?} {} = {}",
        e.event_id(),
        s,
        e.flags(),
        b
    );

    let text = e.string();
    fslog!(fs::switch_log_level_t::SWITCH_LOG_INFO, "\n{}", text);
}

unsafe extern "C" fn zrapi_api(
    cmd: *const std::os::raw::c_char,
    _session: *mut fs::switch_core_session,
    stream: *mut fs::switch_stream_handle_t,
) -> fs::switch_status_t {
    (*stream).write_function.unwrap()(stream, CString::new("OK").unwrap().into_raw());
    let data = std::ffi::CStr::from_ptr(cmd).to_str().unwrap_or("");
    fslog!(
        fs::switch_log_level_t::SWITCH_LOG_INFO,
        "zrapi data: {}",
        data
    );
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrapi_mod_load(mod_int: &fsr::ModInterface) -> fs::switch_status_t {
    mod_int.add_api("zrapi", "zrapi", "zrapi", Some(zrapi_api));
    let id = fsr::event_bind(
        MODULE_NAME,
        fs::switch_event_types_t::SWITCH_EVENT_HEARTBEAT,
        None,
        heartbeat_binding,
    );

    GLOBALS.lock().unwrap().event_id = id;
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrapi_mod_shutdown() -> fs::switch_status_t {
    fsr::event_unbind(GLOBALS.lock().unwrap().event_id);
    thread::sleep(time::Duration::from_millis(100));
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrapi_mod_runtime() -> fs::switch_status_t {
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fsr_export_mod!(
   mod_zrapi_module_interface, MODULE_NAME,
   zrapi_mod_load,
   zrapi_mod_runtime,
   zrapi_mod_shutdown
);
