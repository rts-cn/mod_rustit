use fsr::*;
use lazy_static::lazy_static;
use std::{ffi::CString, sync::Mutex, time::Duration};
pub mod zrs;

#[derive(Debug, Clone)]
struct Binding {
    name: String,
    url: String,
    bindings: String,
    timeout: u64,
    client: reqwest::blocking::Client,
    debug: bool,
}

impl Binding {
    fn new() -> Binding {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        let build = reqwest::blocking::Client::builder().default_headers(headers);
        let client = build.build().unwrap();
        Binding {
            client,
            name: String::from(""),
            url: String::from(""),
            bindings: String::from(""),
            timeout: 0,
            debug: false,
        }
    }
}

struct ZrsModule {
    event_bind_nodes: Vec<u64>,
    xml_bind_node: u64,
    listen_ip: String,
    listen_port: u16,
    password: String,
    apply_inbound_acl: String,
    bindings: Option<Binding>,
}

impl ZrsModule {
    fn new() -> ZrsModule {
        ZrsModule {
            event_bind_nodes: Vec::new(),
            listen_ip: String::from("0.0.0.0"),
            listen_port: 8202,
            password: "".to_string(),
            apply_inbound_acl: "".to_string(),
            xml_bind_node: 0,
            bindings: None,
        }
    }
    fn on_event_bind(id: u64) {
        MODULE.lock().unwrap().event_bind_nodes.push(id);
    }
    fn on_xml_bind_search(id: u64) {
        MODULE.lock().unwrap().xml_bind_node = id;
    }
    fn shutdown() {
        loop {
            let id = MODULE.lock().unwrap().event_bind_nodes.pop();
            let id = id.unwrap_or(0);
            if id > 0 {
                fsr::event_unbind(id);
            } else {
                break;
            }
        }
        let binging = MODULE.lock().unwrap().xml_bind_node;
        if binging > 0 {
            fsr::xml_unbind_search(binging);
        }
        zrs::shutdown();
    }
}

const MODULE_NAME: &str = "mod_zrs";

lazy_static! {
    static ref MODULE: Mutex<ZrsModule> = Mutex::new(ZrsModule::new());
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
                MODULE.lock().unwrap().listen_ip = val;
            } else if var.eq_ignore_ascii_case("listen-port") {
                MODULE.lock().unwrap().listen_port = val.parse::<u16>().unwrap_or(8202);
            } else if var.eq_ignore_ascii_case("password") {
                MODULE.lock().unwrap().password = val;
            } else if var.eq_ignore_ascii_case("apply-inbound-acl") {
                MODULE.lock().unwrap().apply_inbound_acl = val;
            }
            param = (*param).next;
        }

        let tmp_str = CString::new("bindings").unwrap();
        let bindings_tag = switch_xml_child(cfg, tmp_str.as_ptr());
        if bindings_tag.is_null() {
            error!("Missing <bindings> tag!\n");
            fsr::switch_xml_free(xml);
            return;
        }

        let mut binding: Binding = Binding::new();
        let tmp_str = CString::new("binding").unwrap();
        let mut binding_tag = fsr::switch_xml_child(bindings_tag, tmp_str.as_ptr());
        while !binding_tag.is_null() {
            let tmp_str = CString::new("name").unwrap();
            let bname = switch_xml_attr_soft(binding_tag, tmp_str.as_ptr());
            binding.name = to_string(bname);

            let tmp_str = CString::new("param").unwrap();
            let mut param = fsr::switch_xml_child(binding_tag, tmp_str.as_ptr());
            while !param.is_null() {
                let tmp_str = CString::new("name").unwrap();
                let var = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());
                let tmp_str = CString::new("value").unwrap();
                let val = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());

                let var = fsr::to_string(var);
                let val = fsr::to_string(val);

                if var.eq_ignore_ascii_case("gateway-url") {
                    binding.url = val;
                    let tmp_str = CString::new("bindings").unwrap();
                    let bind_mask = switch_xml_attr_soft(param, tmp_str.as_ptr());
                    binding.bindings = to_string(bind_mask);
                } else if var.eq_ignore_ascii_case("timeout") {
                    binding.timeout = val.parse::<u64>().unwrap_or(20);
                    if binding.timeout < 20 {
                        binding.timeout = 20;
                    }
                    if binding.timeout > 120 {
                        binding.timeout = 60;
                    }
                } else if var.eq_ignore_ascii_case("debug") {
                    if val.eq_ignore_ascii_case("on")
                        || val.eq_ignore_ascii_case("yes")
                        || !val.eq_ignore_ascii_case("0")
                        || val.eq_ignore_ascii_case("true")
                    {
                        binding.debug = true;
                    } else {
                        binding.debug = false;
                    }
                }
                param = (*param).next;
            }
            binding_tag = (*binding_tag).next;
        }

        MODULE.lock().unwrap().bindings = Some(binding);
        fsr::switch_xml_free(xml);
    }
}

fn xml_fetch(data: String) -> String {
    let binding = MODULE.lock().unwrap().bindings.clone();
    match binding {
        None => (),
        Some(binding) => {
            let mut request = String::new();
            if binding.debug {
                request = data.clone();
            }
            let response = binding
                .client
                .post(binding.url)
                .timeout(Duration::from_secs(binding.timeout))
                .body(data)
                .send();
            match response {
                Ok(response) => {
                    let text = response.text();
                    match text {
                        Ok(text) => {
                            if binding.debug {
                                debug!("XML Fetch:\n{}\n{}", request, text);
                            }
                            if !text.is_empty() {
                                return text;
                            }
                            warn!("XML Fetch recv empty response!!!");
                        }
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
    }
    String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
    <document type="freeswitch/xml">
      <section name="result">
        <result status="not found"/>
      </section>
    </document>"#,
    )
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

    ZrsModule::on_event_bind(id);

    let listen_ip = MODULE.lock().unwrap().listen_ip.clone();
    let listen_port = MODULE.lock().unwrap().listen_port;
    let bind_uri = format!("{}:{:?}", listen_ip, listen_port);
    let password = MODULE.lock().unwrap().password.clone();
    let acl = MODULE.lock().unwrap().apply_inbound_acl.clone();

    zrs::serve(bind_uri, password, acl);

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

    let binding = MODULE.lock().unwrap().bindings.clone();
    match binding {
        None => (),
        Some(binding) => {
            notice!(
                "Binding [{}] XML Fetch Function [{}] [{}]\n",
                binding.name,
                binding.url,
                binding.bindings
            );
            let binding = xml_bind_search(&binding.bindings, xml_fetch);
            ZrsModule::on_xml_bind_search(binding);
        }
    }
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_shutdown() -> switch_status_t {
    ZrsModule::shutdown();
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fn zrs_mod_runtime() -> switch_status_t {
    switch_status_t::SWITCH_STATUS_SUCCESS
}

fsr_mod!("mod_zrs", zrs_mod_load, zrs_mod_runtime, zrs_mod_shutdown);
