#[macro_use]
extern crate libc;
use std::{borrow::Cow, sync::Mutex, thread, time};

pub mod fsr;
use fsr::*;

use lazy_static::lazy_static;

struct Global {
    event_id: u64,
}

impl Global {
    fn new() -> Global {
        Global { event_id: 0 }
    }
}

lazy_static! {
    static ref GLOBALS: Mutex<Global> = Mutex::new(Global::new());
}

fn example_binding(e: fsr::Event) {
    let s = e.subclass_name().unwrap_or(Cow::Borrowed("None"));
    let b = e.body().unwrap_or(Cow::Borrowed("<No Body>"));
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

fn mod_load(mod_int: &fsr::ModInterface) -> fs::switch_status_t {
    mod_int.add_raw_api("zrapi", "Example doc", "zrapi", zrapi_api);
    let id = fsr::event_bind(
        MOD_ZRAPI_DEF.name,
        fs::switch_event_types_t::SWITCH_EVENT_HEARTBEAT,
        None,
        example_binding,
    );

    GLOBALS.lock().unwrap().event_id = id;
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fn mod_unload() -> fs::switch_status_t {
    fsr::event_unbind(GLOBALS.lock().unwrap().event_id);
    thread::sleep(time::Duration::from_millis(100));
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

static MOD_ZRAPI_DEF: fsr::ModDefinition = fsr::ModDefinition {
    name: "mod_zrapi",
    load: mod_load,
    shutdown: Some(mod_unload),
    runtime: None,
};

fsr_export_mod!(mod_zrapi_module_interface, MOD_ZRAPI_DEF);

#[allow(unused_variables)]
unsafe extern "C" fn zrapi_api(
    cmd: *const std::os::raw::c_char,
    session: *mut fs::switch_core_session,
    stream: *mut fs::switch_stream_handle_t,
) -> fs::switch_status_t {
    (*stream).write_function.unwrap()(stream, fs::str_to_ptr("OK"));
    let data = std::ffi::CStr::from_ptr(cmd).to_str().unwrap_or("");
    fslog!(
        fs::switch_log_level_t::SWITCH_LOG_INFO,
        "Logging data: {}",
        data
    );
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}
