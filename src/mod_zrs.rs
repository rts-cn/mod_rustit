use fsr::*;
use std::{ffi::CString, thread};
use tokio::runtime::Runtime;
pub mod cdr;
pub mod api;
pub mod storage;
pub mod xml;

const MODULE_NAME: &str = "mod_zrs";

fn api_zsr(_session: &fsr::Session, cmd: String, stream: &fsr::Stream) -> fsr::switch_status_t {
    debug!("api zsr:{}", cmd);
    stream.write("OK");
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn app_zsr(_session: &fsr::Session, cmd: String) {
    debug!("api zsr:{}", cmd);
}

fn do_config() {
    let cf = "zrs.conf";
    let mut cfg: fsr::switch_xml_t = std::ptr::null_mut();
    unsafe {
        let event = std::ptr::null_mut();
        let tmp_str = CString::new(cf).unwrap();
        let xml = fsr::switch_xml_open_cfg(tmp_str.as_ptr(), std::ptr::addr_of_mut!(cfg), event);
        if xml.is_null() {
            error!("open of {} failed", cf);
            fsr::switch_xml_free(xml);
            return;
        }
        api::load_config(cfg);
        xml::load_config(cfg);
        cdr::load_config(cfg);
        storage::load_config(cfg);
        fsr::switch_xml_free(xml);
    }
}

fn zrs_mod_load(m: &fsr::Module) -> switch_status_t {
    do_config();
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

    xml::start();
    cdr::start();
    storage::start(m, MODULE_NAME);
    api::start(m, MODULE_NAME);
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_shutdown() -> switch_status_t {
    api::shutdown();
    xml::shutdown();
    cdr::shutdown();
    storage::shutdown();

    let rt = Runtime::new().unwrap();
    rt.shutdown_timeout(tokio::time::Duration::from_millis(1000));
    thread::sleep(std::time::Duration::from_millis(200));
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_runtime() -> switch_status_t {
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fsr_mod!("mod_zrs", zrs_mod_load, zrs_mod_runtime, zrs_mod_shutdown);
