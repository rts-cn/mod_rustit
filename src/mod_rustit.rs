use switch_sys::*;
use std::{ffi::CString, thread};
use tokio::runtime::Runtime;
pub mod cdr;
pub mod grcp;
pub mod storage;
pub mod xml;
pub mod api;

const MODULE_NAME: &str = "mod_rustit";

fn api_rustit(_session: &switch_sys::Session, cmd: String, stream: &switch_sys::Stream) -> switch_sys::switch_status_t {
    debug!("api rustit:{}", cmd);
    stream.write("OK");
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn app_rustit(_session: &switch_sys::Session, cmd: String) {
    debug!("api rustit:{}", cmd);
}

fn do_config() {
    let cf = "rustit.conf";
    let mut cfg: switch_sys::switch_xml_t = std::ptr::null_mut();
    unsafe {
        let event = std::ptr::null_mut();
        let tmp_str = CString::new(cf).unwrap();
        let xml = switch_sys::switch_xml_open_cfg(tmp_str.as_ptr(), std::ptr::addr_of_mut!(cfg), event);
        if xml.is_null() {
            error!("open of {} failed", cf);
            switch_sys::switch_xml_free(xml);
            return;
        }
        grcp::load_config(cfg);
        xml::load_config(cfg);
        cdr::load_config(cfg);
        storage::load_config(cfg);
        api::load_config(cfg);
        switch_sys::switch_xml_free(xml);
    }
}

fn zrs_mod_load(m: &switch_sys::Module) -> switch_status_t {
    do_config();
    fsr_api!(m, "rustit", "rustit desc", "rustit syntax", api_rustit);
    fsr_app!(
        m,
        "rustit",
        "rustit short desc",
        "rustit long desc",
        "rustit syntax",
        app_rustit,
        switch_application_flag_enum_t::SAF_NONE
    );

    xml::start();
    cdr::start();
    storage::start(m, MODULE_NAME);
    grcp::start(m, MODULE_NAME);
    api::start(m, MODULE_NAME);
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_shutdown() -> switch_status_t {
    grcp::shutdown();
    xml::shutdown();
    cdr::shutdown();
    storage::shutdown();
    api::shutdown();

    let rt = Runtime::new().unwrap();
    rt.shutdown_timeout(tokio::time::Duration::from_millis(1000));
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_runtime() -> switch_status_t {
    thread::sleep(std::time::Duration::from_millis(1));
    switch_status_t::SWITCH_STATUS_TERM
}

fsr_mod!("mod_rustit", zrs_mod_load, zrs_mod_runtime, zrs_mod_shutdown);
