use lazy_static::lazy_static;
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

fn on_event(e: fsr::Event) {
    let event = zrs::Event::from(&e);
    let _ = zrs::broadcast(event);
}

fn api_zsr(_session: &fsr::Session, cmd: String, stream: &fsr::Stream) -> fsr::switch_status_t {
    debug!("api zsr:{}", cmd);
    stream.write("OK");
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn app_zsr(_session: &fsr::Session, cmd: String) {
    debug!("api zsr:{}", cmd);
}

fn zrs_mod_load(m: &fsr::Module) -> switch_status_t {
    let id = fsr::event_bind(
        m,
        MODULE_NAME,
        switch_event_types_t::SWITCH_EVENT_ALL,
        None,
        on_event,
    );

    Global::save_node(id);

    let addr = "0.0.0.0:8208";
    info!("Listen and serve: {}", addr);

    zrs::serve(addr.to_string());

    fsr_api!(m, "zsr", "zsr desc", "zsr syntax", api_zsr);

    fsr_app!(
        m,
        "zsr",
        "zsr short desc",
        "zsr long desc",
        "zsr syntax",
        app_zsr,
        switch_application_flag_enum_t::SAF_NONE
    );

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

fsr_mod!("mod_zrs", zrs_mod_load, zrs_mod_runtime, zrs_mod_shutdown);
