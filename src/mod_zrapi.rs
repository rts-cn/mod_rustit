use lazy_static::lazy_static;
use std::ffi::CString;
use std::sync::Mutex;

pub mod fsr;
use fsr::*;

struct Global {
    ev_nodes: Vec<u64>,
}

impl Global {
    fn new() -> Global {
        Global {
            ev_nodes: Vec::new(),
        }
    }
    fn save_node(& mut self, id:u64) {
        self.ev_nodes.push(id);
    }
}

const MODULE_NAME: &str = "mod_zrapi";

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

    let text = e.json();
    fslog!(fs::switch_log_level_t::SWITCH_LOG_INFO, "\n{}", text);
}

unsafe extern "C" fn zrapi_api(
    cmd: *const std::os::raw::c_char,
    _session: *mut fs::switch_core_session,
    stream: *mut fs::switch_stream_handle_t,
) -> fs::switch_status_t {
    let ok = CString::new("OK").expect("CString::new failed");
    (*stream).write_function.unwrap()(stream, ok.as_ptr());
    let data = std::ffi::CStr::from_ptr(cmd).to_string_lossy().to_string();
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
        mod_int,
        MODULE_NAME,
        fs::switch_event_types_t::SWITCH_EVENT_HEARTBEAT,
        None,
        heartbeat_binding,
    );

    GLOBALS.lock().unwrap().save_node(id);
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrapi_mod_shutdown() -> fs::switch_status_t {
    let ev_nodes = &GLOBALS.lock().unwrap().ev_nodes;
    for id in ev_nodes {
        fsr::event_unbind(*id);
    }
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrapi_mod_runtime() -> fs::switch_status_t {
    fs::switch_status_t::SWITCH_STATUS_SUCCESS
}

fsr_export_mod!(
    mod_zrapi_module_interface,
    MODULE_NAME,
    zrapi_mod_load,
    zrapi_mod_runtime,
    zrapi_mod_shutdown
);
