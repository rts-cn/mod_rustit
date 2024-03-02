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
    re: regex::Regex,
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
        let build = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .use_rustls_tls();
        let client = build.build().unwrap();
        Binding {
            client,
            re: regex::Regex::new(
                r#"(?i)<X-PRE-PROCESS\s+cmd\s*=\s*"(set|env\-set|exec\-set|stun\-set|include|exec)"\s+data\s*=\s*"(.+)"\s*/>"#,
            )
            .unwrap(),
            name: String::from(""),
            url: String::from(""),
            bindings: String::from(""),
            timeout: 0,
            debug: false,
        }
    }
}

mod pre {
    use fsr::*;
    fn expand_vars(s: &str) -> String {
        let mut expand = String::from(s);
        for (pos, _) in s.match_indices("$${") {
            let end = (s[pos..]).to_string().find("}");
            if let Some(end) = end {
                let vname = s[pos + 3..end + pos].to_string();
                let val = fsr::get_variable(&vname);
                expand = expand.replace(&format!("$${{{}}}", vname), &val);
            }
        }
        expand
    }

    fn set(data: &str) {
        let r = data.split_once("=");
        match r {
            Some((name, val)) => {
                if !name.is_empty() && !val.is_empty() {
                    fsr::set_variable(name, val);
                }
            }
            None => {}
        }
    }

    fn env_set(data: &str) {
        let r = data.split_once("=");
        match r {
            Some((name, val)) => {
                if !name.is_empty() && !val.is_empty() {
                    info!("name { }, val {}", name, val);
                }
            }
            None => {}
        }
    }

    fn stun_set(data: &str) {
        let r = data.split_once("=");
        match r {
            Some((name, val)) => {
                if !name.is_empty() && !val.is_empty() {
                    info!("name { }, val {}", name, val);
                }
            }
            None => {}
        }
    }

    pub fn process(re: regex::Regex, text: std::borrow::Cow<'_, str>) {
        if text.contains("X-PRE-PROCESS") {
            return;
        }
        for line in text.lines() {
            for cap in re.captures_iter(&line) {
                let (full, [cmd, data]) = cap.extract();
                let expand = expand_vars(data);
                if cmd.eq_ignore_ascii_case("set") {
                    set(&expand)
                } else if cmd.eq_ignore_ascii_case("stun-set") {
                    stun_set(&expand)
                } else if cmd.eq_ignore_ascii_case("env-set") {
                    env_set(&expand)
                } else {
                    warn!("Unsupported pre process command {}", full);
                }
            }
        }
    }
}

fn xml_fetch(data: String) -> Vec<u8> {
    let error = Vec::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<document type="freeswitch/xml">
<section name="result">
<result status="not found"/>
</section>
</document>"#,
    );

    if GOLOBAS.read().unwrap().running == false {
        return error;
    }

    let binding = &GOLOBAS.read().unwrap().bindings;
    if let Some(binding) = binding {
        let mut request = "".to_string();
        if binding.debug {
            request = data.clone();
        }
        let client = binding.client.clone();
        let response = client
            .post(&binding.url)
            .timeout(Duration::from_millis(binding.timeout))
            .body(data)
            .send();
        match response {
            Ok(response) => {
                let body = response.bytes();
                match body {
                    Ok(body) => {
                        let text: std::borrow::Cow<'_, str> =
                            String::from_utf8_lossy(body.as_ref());

                        if binding.debug {
                            debug!("XML Fetch:\n{}\n{}", request, text);
                        }

                        pre::process(binding.re.clone(), text);

                        if body.len() > 0 {
                            return body.to_vec();
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

    return error;
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
    lazy_static::initialize(&GOLOBAS);
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
                    binding.timeout = val.parse::<u64>().unwrap_or(5000);
                    if binding.timeout < 1000 {
                        binding.timeout = 1000;
                    }
                    if binding.timeout > 60000 {
                        binding.timeout = 60000;
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
    GOLOBAS.write().unwrap().running = false;
    let binging = GOLOBAS.read().unwrap().bind_node;
    if binging > 0 {
        debug!("unbind xml search");
        fsr::xml_unbind_search(binging);
    }
}
