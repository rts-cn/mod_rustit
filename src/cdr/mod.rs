use switch_sys::*;
use lazy_static::lazy_static;

mod cdr;

use std::{ffi::CString, sync::RwLock};

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub format: String,
    pub url: String,
    pub log_dir: String,
    pub err_log_dir: String,
    pub log_b_leg: bool,
    pub prefix_a_leg: bool,
    pub log_http_and_disk: bool,
    pub log_errors_to_disk: bool,
    pub timeout: u64,
    pub retries: i32,
    pub delay: i32,
    pub encode_values: bool,
    pub client: reqwest::blocking::Client,
}
impl Profile {
    pub fn new() -> Profile {
        let build = reqwest::blocking::Client::builder().use_rustls_tls();
        let client = build.build().unwrap();
        Profile {
            client,
            name: String::new(),
            url: String::new(),
            format: String::new(),
            log_dir: String::new(),
            err_log_dir: String::new(),
            log_b_leg: false,
            prefix_a_leg: false,
            log_http_and_disk: false,
            log_errors_to_disk: true,
            timeout: 60,
            retries: 0,
            delay: 5,
            encode_values: true,
        }
    }
}

struct Global {
    running: bool,
    state_handlers: usize,
    profile: Option<Profile>,
}
impl Global {
    pub fn new() -> Global {
        Global {
            state_handlers: 0,
            running: false,
            profile: None,
        }
    }
}

unsafe extern "C" fn on_reporting(session: *mut switch_core_session_t) -> switch_status_t {
    let profile = GOLOBAS.read().unwrap().profile.clone();
    match profile {
        None => switch_status_t::SWITCH_STATUS_SUCCESS,
        Some(profile) => {
            let cdr = cdr::generate_cdr(&profile, session);
            match cdr {
                Ok(cdr) => {
                    cdr::process_cdr(profile, cdr);
                    switch_status_t::SWITCH_STATUS_SUCCESS
                }
                Err(status) => status,
            }
        }
    }
}

lazy_static! {
    static ref GOLOBAS: RwLock<Global> = RwLock::new(Global::new());
}

pub fn start() {
    let cdr_profile = GOLOBAS.read().unwrap().profile.clone();
    if let Some(cdr_profile) = cdr_profile {
        notice!(
            "Add CDR handler [{}] [{}] [{}]",
            cdr_profile.name,
            cdr_profile.url,
            cdr_profile.format
        );

        let mut state_handlers = Box::new(switch_state_handler_table_t::default());
        state_handlers.on_reporting = Some(on_reporting);
        let state_handlers_ptr = Box::leak(state_handlers) as *const switch_state_handler_table_t;
        unsafe { switch_core_add_state_handler(state_handlers_ptr) };
        GOLOBAS.write().unwrap().running = true;
        GOLOBAS.write().unwrap().state_handlers =
            state_handlers_ptr as *const _ as *const usize as usize;
    }
}

pub fn shutdown() {
    let running = GOLOBAS.read().unwrap().running;
    if running {
        debug!("remove cdr report state handler");
        let state_handlers = GOLOBAS.read().unwrap().state_handlers;
        let state_handlers = state_handlers as *mut usize;
        let state_handlers = state_handlers as *mut _ as *mut switch_state_handler_table_t;
        unsafe {
            switch_core_remove_state_handler(state_handlers);
            let _ = Box::from_raw(state_handlers);
        };
    }
}

pub fn load_config(cfg: switch_xml_t) {
    lazy_static::initialize(&GOLOBAS);
    unsafe {
        let tmp_str = CString::new("cdrs").unwrap();
        let cdrs_tag = switch_xml_child(cfg, tmp_str.as_ptr());
        if cdrs_tag.is_null() {
            warn!("Missing <cdrs> tag!");
            return;
        }

        let mut cdr_profile = Profile::new();
        let tmp_str = CString::new("cdr").unwrap();
        let mut cdr_tag = switch_sys::switch_xml_child(cdrs_tag, tmp_str.as_ptr());
        while !cdr_tag.is_null() {
            let tmp_str = CString::new("name").unwrap();
            let bname = switch_xml_attr_soft(cdr_tag, tmp_str.as_ptr());
            cdr_profile.name = switch_to_string(bname);

            let tmp_str = CString::new("param").unwrap();
            let mut param = switch_sys::switch_xml_child(cdr_tag, tmp_str.as_ptr());
            while !param.is_null() {
                let tmp_str = CString::new("name").unwrap();
                let var = switch_sys::switch_xml_attr_soft(param, tmp_str.as_ptr());
                let tmp_str = CString::new("value").unwrap();
                let val = switch_sys::switch_xml_attr_soft(param, tmp_str.as_ptr());

                let var = switch_sys::switch_to_string(var);
                let val = switch_sys::switch_to_string(val);

                if var.eq_ignore_ascii_case("url") {
                    cdr_profile.url = val;
                } else if var.eq_ignore_ascii_case("format") {
                    cdr_profile.format = val;
                } else if var.eq_ignore_ascii_case("retries") {
                    cdr_profile.retries = val.parse::<i32>().unwrap_or(1);
                    if cdr_profile.retries < 1 {
                        cdr_profile.retries = 1;
                    }
                    if cdr_profile.retries > 10 {
                        cdr_profile.retries = 10;
                    }
                } else if var.eq_ignore_ascii_case("delay") {
                    cdr_profile.delay = val.parse::<i32>().unwrap_or(5);
                    if cdr_profile.delay < 1 {
                        cdr_profile.delay = 1;
                    }
                    if cdr_profile.delay > 120 {
                        cdr_profile.delay = 120;
                    }
                } else if var.eq_ignore_ascii_case("log-http-and-disk") {
                    cdr_profile.log_http_and_disk = switch_sys::switch_true(&val);
                } else if var.eq_ignore_ascii_case("log-dir") {
                    if val.is_empty() {
                        cdr_profile.log_dir = format!("{}/zrs_cdr", get_variable("log_dir"));
                    } else {
                        cdr_profile.log_dir = val;
                    }
                } else if var.eq_ignore_ascii_case("log-b-leg") {
                    cdr_profile.log_b_leg = switch_sys::switch_true(&val);
                } else if var.eq_ignore_ascii_case("prefix-a-leg") {
                    cdr_profile.prefix_a_leg = switch_sys::switch_true(&val);
                } else if var.eq_ignore_ascii_case("err-log-dir") {
                    if val.is_empty() {
                        cdr_profile.err_log_dir = format!("{}/zrs_cdr", get_variable("log_dir"));
                    } else {
                        cdr_profile.err_log_dir = val;
                    }
                } else if var.eq_ignore_ascii_case("timeout") {
                    cdr_profile.timeout = val.parse::<u64>().unwrap_or(5000);
                    if cdr_profile.timeout < 1000 {
                        cdr_profile.timeout = 1000;
                    }
                    if cdr_profile.timeout > 6000 {
                        cdr_profile.timeout = 6000;
                    }
                } else if var.eq_ignore_ascii_case("encode-values") {
                    cdr_profile.encode_values = switch_true(&val);
                }
                param = (*param).next;
            }
            cdr_tag = (*cdr_tag).next;
        }

        if cdr_profile.url.starts_with("http://") || cdr_profile.url.starts_with("https://") {
            GOLOBAS.write().unwrap().profile = Some(cdr_profile);
        }
    }
}
