use fsr::*;
use lazy_static::lazy_static;
use std::{ffi::CString, sync::RwLock};
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct Binding {
    pub name: String,
    pub url: String,
    pub bindings: String,
    pub timeout: u64,
    pub client: reqwest::blocking::Client,
    pub debug: bool,
}

struct Global {
    bind_node: u64,
    running: bool,
    bindings: Option<Binding>,
}
impl Global {
    pub fn new() -> Global {
        Global {
            bind_node: 0,
            running: false,
            bindings: None,
        }
    }
}

lazy_static! {
    static ref GOLOBAS: RwLock<Global> = RwLock::new(Global::new());
}

impl Binding {
    pub fn new() -> Binding {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        let build = reqwest::blocking::Client::builder().default_headers(headers).use_rustls_tls();
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

fn xml_fetch(data: String) -> String {
    let binding = GOLOBAS.read().unwrap().bindings.clone();
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

pub fn start() {
    let binding = GOLOBAS.read().unwrap().bindings.clone();
    match binding {
        None => (),
        Some(binding) => {
            notice!(
                "Binding [{}] XML Fetch Function [{}] [{}]",
                binding.name,
                binding.url,
                binding.bindings
            );
            let binding = xml_bind_search(&binding.bindings, xml_fetch);
            GOLOBAS.write().unwrap().running = true;
            GOLOBAS.write().unwrap().bind_node = binding;
        }
    }
}

pub fn load_config(cfg: switch_xml_t) {
    unsafe {
        let tmp_str = CString::new("bindings").unwrap();
        let bindings_tag = switch_xml_child(cfg, tmp_str.as_ptr());
        if bindings_tag.is_null() {
            warn!("Missing <bindings> tag!");
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
                    binding.debug = fsr::switch_true(&val);
                }
                param = (*param).next;
            }
            binding_tag = (*binding_tag).next;
        }

        if binding.url.starts_with("http://") || binding.url.starts_with("https://") {
            GOLOBAS.write().unwrap().bindings = Some(binding);
        }
    }
}

pub fn shutdown() {
    let binging = GOLOBAS.read().unwrap().bind_node;
    if binging > 0 {
        fsr::xml_unbind_search(binging);
    }
}
