use lazy_static::lazy_static;
use std::ffi::CString;
use std::sync::Mutex;

use fsr::*;

pub mod zrs;

struct Global {
    enode: Vec<u64>,
}

impl Global {
    fn new() -> Global {
        Global { enode: Vec::new() }
    }
    fn save_node(id: u64) {
        GLOBALS.lock().unwrap().enode.push(id);
    }

    fn remove_node() {
        loop {
            let id = GLOBALS.lock().unwrap().enode.pop();
            let id = id.unwrap_or(0);
            if id > 0 {
                fsr::event_unbind(id);
            } else {
                break;
            }
        }
    }
}

const MODULE_NAME: &str = "mod_zrs";

lazy_static! {
    static ref GLOBALS: Mutex<Global> = Mutex::new(Global::new());
}

fn heartbeat_binding(e: fsr::Event) {
    let event = zrs::Event::from(&e);
    let ev_string = serde_json::to_string(&event).unwrap();
    fslog!(
        fs::switch_log_level_t::SWITCH_LOG_DEBUG,
        "broadcast event:\n{}",
        ev_string
    );

    let _ = zrs::broadcast(event);
    fslog!(
        fs::switch_log_level_t::SWITCH_LOG_INFO,
        "The Event has been broadcast"
    );
}

unsafe extern "C" fn zrs_api(
    cmd: *const std::os::raw::c_char,
    _session: *mut fs::switch_core_session,
    stream: *mut fs::switch_stream_handle_t,
) -> fs::switch_status_t {
    let ok = CString::new("OK").expect("CString::new failed");
    (*stream).write_function.unwrap()(stream, ok.as_ptr());
    let data = std::ffi::CStr::from_ptr(cmd).to_string_lossy().to_string();
    fslog!(
        fs::switch_log_level_t::SWITCH_LOG_INFO,
        "zrs data: {}",
        data
    );

    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_load(mod_int: &fsr::ModInterface) -> fs::switch_status_t {
    mod_int.add_api("zrs", "zrs", "zrs", Some(zrs_api));
    let id = fsr::event_bind(
        mod_int,
        MODULE_NAME,
        fs::switch_event_types_t::SWITCH_EVENT_HEARTBEAT,
        None,
        heartbeat_binding,
    );

    Global::save_node(id);

    let addr = "0.0.0.0:8208";
    fslog!(
        fs::switch_log_level_t::SWITCH_LOG_NOTICE,
        "Listen and serve: {}",
        addr
    );

    zrs::serve(addr.to_string());

    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_shutdown() -> fs::switch_status_t {
    Global::remove_node();
    zrs::shutdown();
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_runtime() -> fs::switch_status_t {
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fsr_export_mod!(
    mod_zrs_module_interface,
    MODULE_NAME,
    zrs_mod_load,
    zrs_mod_runtime,
    zrs_mod_shutdown
);
