use fsr::*;
use lazy_static::lazy_static;
use std::{ffi::CString, sync::RwLock, thread};
pub mod cdr_post;
pub mod xml_fetch;
pub mod zrs;

struct ZrsModule {
    event_bind_nodes: Vec<u64>,
    listen_ip: String,
    listen_port: u16,
    password: String,
    apply_inbound_acl: String,
    enable: bool,
}

impl ZrsModule {
    fn new() -> ZrsModule {
        ZrsModule {
            event_bind_nodes: Vec::new(),
            listen_ip: String::from("0.0.0.0"),
            listen_port: 8202,
            password: "".to_string(),
            apply_inbound_acl: "".to_string(),
            enable: false,
        }
    }
    fn on_event_bind(id: u64) {
        MODULE.write().unwrap().event_bind_nodes.push(id);
    }
    fn shutdown() {
        xml_fetch::shutdown();
        cdr_post::shutdown();
        loop {
            let id = MODULE.write().unwrap().event_bind_nodes.pop();
            match id {
                Some(id) => {
                    debug!("event unbind");
                    fsr::event_unbind(id);
                }
                None => {
                    break;
                }
            }
        }
        if MODULE.read().unwrap().enable {
            zrs::shutdown();
        }
    }
}

const MODULE_NAME: &str = "mod_zrs";

lazy_static! {
    static ref MODULE: RwLock<ZrsModule> = RwLock::new(ZrsModule::new());
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
            error!("open of {} failed", cf);
            fsr::switch_xml_free(xml);
            return;
        }

        let tmp_str = CString::new("grpc").unwrap();
        let settings_tag = fsr::switch_xml_child(cfg, tmp_str.as_ptr());
        if !settings_tag.is_null() {
            let tmp_str = CString::new("param").unwrap();
            let mut param = fsr::switch_xml_child(settings_tag, tmp_str.as_ptr());
            while !param.is_null() {
                let tmp_str = CString::new("name").unwrap();
                let var = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());
                let tmp_str = CString::new("value").unwrap();
                let val = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());

                let var = fsr::to_string(var);
                let val = fsr::to_string(val);

                let mut m = MODULE.write().unwrap();
                if var.eq_ignore_ascii_case("listen-ip") {
                    m.listen_ip = val;
                } else if var.eq_ignore_ascii_case("listen-port") {
                    m.listen_port = val.parse::<u16>().unwrap_or(8202);
                } else if var.eq_ignore_ascii_case("password") {
                    m.password = val;
                } else if var.eq_ignore_ascii_case("apply-inbound-acl") {
                    m.apply_inbound_acl = val;
                } else if var.eq_ignore_ascii_case("enable") {
                    m.enable = switch_true(&val);
                }

                drop(m);
                param = (*param).next;
            }
        }

        xml_fetch::load_config(cfg);
        cdr_post::load_config(cfg);
        fsr::switch_xml_free(xml);
    }
}

fn zrs_mod_load(m: &fsr::Module) -> switch_status_t {
    do_config();

    if MODULE.read().unwrap().enable {
        let id = fsr::event_bind(
            m,
            MODULE_NAME,
            switch_event_types_t::SWITCH_EVENT_ALL,
            None,
            on_event,
        );

        ZrsModule::on_event_bind(id);

        let module = MODULE.read().unwrap();
        let listen_ip = module.listen_ip.clone();
        let listen_port = module.listen_port;
        let bind_uri = format!("{}:{:?}", listen_ip, listen_port);
        let password = module.password.clone();
        let acl = module.apply_inbound_acl.clone();
        drop(module);
        zrs::serve(bind_uri, password, acl);
    }

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

    xml_fetch::start();
    cdr_post::start();

    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_shutdown() -> switch_status_t {
    ZrsModule::shutdown();
    // Wait for tokio runtime shutdown
    thread::sleep(std::time::Duration::from_millis(200));
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_runtime() -> switch_status_t {
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fsr_mod!("mod_zrs", zrs_mod_load, zrs_mod_runtime, zrs_mod_shutdown);
