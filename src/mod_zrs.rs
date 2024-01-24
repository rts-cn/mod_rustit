use lazy_static::lazy_static;
use std::{ffi::CString, sync::Mutex};

use fsr::*;
pub mod zrs;

struct Global {
    enode: Vec<u64>,
    listen_ip: String,
    listen_port: u16,
    gateway_url: String,
    timeout: u32,
    secret_key: String,
    apply_inbound_acl: String,
}

impl Global {
    fn new() -> Global {
        Global {
            enode: Vec::new(),
            listen_ip: String::from("0.0.0.0"),
            listen_port: 8202,
            gateway_url: String::from(""),
            timeout: 10,
            secret_key: "".to_string(),
            apply_inbound_acl: "".to_string(),
        }
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

fn do_config() {
    let cf = "zrs.conf";
    let mut cfg: fsr::switch_xml_t = std::ptr::null_mut();
    unsafe {
        let event = std::ptr::null_mut();
        let tmp_str = CString::new(cf).unwrap();
        let xml = fsr::switch_xml_open_cfg(tmp_str.as_ptr(), std::ptr::addr_of_mut!(cfg), event);
        if xml.is_null() {
            error!("open of {} failed\n", cf);
            fsr::switch_xml_free(xml);
            return;
        }

        let tmp_str = CString::new("settings").unwrap();
        let settings_tag = fsr::switch_xml_child(cfg, tmp_str.as_ptr());
        if settings_tag.is_null() {
            error!("Missing <settings> tag!\n");
            fsr::switch_xml_free(xml);
            return;
        }

        let tmp_str = CString::new("param").unwrap();
        let mut param = fsr::switch_xml_child(settings_tag, tmp_str.as_ptr());
        while !param.is_null() {
            let tmp_str = CString::new("name").unwrap();
            let var = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());
            let tmp_str = CString::new("value").unwrap();
            let val = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());

            let var = fsr::to_string(var);
            let val = fsr::to_string(val);

            if var.eq_ignore_ascii_case("listen-ip") {
                GLOBALS.lock().unwrap().listen_ip = val;
            } else if var.eq_ignore_ascii_case("listen-port") {
                GLOBALS.lock().unwrap().listen_port = val.parse::<u16>().unwrap_or(8202);
            } else if var.eq_ignore_ascii_case("gateway-url") {
                GLOBALS.lock().unwrap().gateway_url = val;
            } else if var.eq_ignore_ascii_case("timeout") {
                GLOBALS.lock().unwrap().timeout = val.parse::<u32>().unwrap_or(10);
            } else if var.eq_ignore_ascii_case("secret_key") {
                GLOBALS.lock().unwrap().secret_key = val;
            } else if var.eq_ignore_ascii_case("apply-inbound-acl") {
                GLOBALS.lock().unwrap().apply_inbound_acl = val;
            }
            param = (*param).next;
        }
        fsr::switch_xml_free(xml);
    }
}

fn zrs_mod_load(m: &fsr::Module) -> switch_status_t {
    do_config();

    let id = fsr::event_bind(
        m,
        MODULE_NAME,
        switch_event_types_t::SWITCH_EVENT_ALL,
        None,
        on_event,
    );

    Global::save_node(id);

    let listen_ip = GLOBALS.lock().unwrap().listen_ip.clone();
    let listen_port = GLOBALS.lock().unwrap().listen_port;

    let dst = GLOBALS.lock().unwrap().gateway_url.clone();
    let secret = GLOBALS.lock().unwrap().secret_key.clone();
    let apply_inbound_acl = GLOBALS.lock().unwrap().apply_inbound_acl.clone();

    let info = zrs::Info {
        name: fsr::get_variable("hostname"),
        ip: fsr::get_variable("local_ip_v4"),
        uuid: fsr::get_variable("core_uuid"),
        uri: format!("http://{}:{}", listen_ip, listen_port),
    };

    let server = zrs::Server {
        bind_uri:format!("{}:{}", listen_ip, listen_port),
        register_uri: dst,
        secret,
        apply_inbound_acl,
    };

    zrs::serve(server, info);

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
