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
    debug!("broadcast event:\n{}", ev_string);

    let _ = zrs::broadcast(event);
    debug!("The Event has been broadcast");
}

unsafe extern "C" fn zrs_api(
    cmd: *const std::os::raw::c_char,
    _session: *mut switch_core_session,
    stream: *mut switch_stream_handle_t,
) -> switch_status_t {
    let ok = CString::new("OK").expect("CString::new failed");
    (*stream).write_function.unwrap()(stream, ok.as_ptr());
    let data = std::ffi::CStr::from_ptr(cmd).to_string_lossy().to_string();
    debug!("zrs data: {}", data);
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_load(m: &fsr::Module) -> switch_status_t {
    m.add_api("zrs", "zrs", "zrs", Some(zrs_api));
    let id = fsr::event_bind(
        m,
        MODULE_NAME,
        switch_event_types_t::SWITCH_EVENT_HEARTBEAT,
        None,
        heartbeat_binding,
    );

    Global::save_node(id);

    let addr = "0.0.0.0:8208";
    info!("Listen and serve: {}", addr);

    zrs::serve(addr.to_string());

    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_shutdown() -> switch_status_t {
    Global::remove_node();
    zrs::shutdown();
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_runtime() -> switch_status_t {
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fsr_mod!(
    mod_zrs_module_interface,
    MODULE_NAME,
    zrs_mod_load,
    zrs_mod_runtime,
    zrs_mod_shutdown
);
