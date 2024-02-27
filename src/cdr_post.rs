use fsr::*;
use lazy_static::lazy_static;
use std::{fs, path::Path};
use tokio::time::Duration;

use std::{
    ffi::{c_char, CString},
    os::raw::c_void,
    sync::RwLock,
    thread,
};

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
    profile: Option<Profile>,
}
impl Global {
    pub fn new() -> Global {
        Global {
            running: false,
            profile: None,
        }
    }
}

#[derive(Debug, Clone)]
struct CdrData {
    fromat: String,
    text: String,
    uuid: String,
    filename: String,
}

static mut STATE_HANDLERS: switch_state_handler_table_t = switch_state_handler_table_t {
    on_init: None,
    on_routing: None,
    on_execute: None,
    on_hangup: None,
    on_exchange_media: None,
    on_soft_execute: None,
    on_consume_media: None,
    on_hibernate: None,
    on_reset: None,
    on_park: None,
    on_reporting: Some(on_reporting),
    on_destroy: None,
    flags: 0,
    padding: [std::ptr::null_mut(); 10],
};

lazy_static! {
    static ref GOLOBAS: RwLock<Global> = RwLock::new(Global::new());
}

pub fn start() {
    let cdr_profile = GOLOBAS.read().unwrap().profile.clone();
    match cdr_profile {
        None => (),
        Some(cdr_profile) => {
            notice!(
                "Add CDR handler [{}] [{}] [{}]",
                cdr_profile.name,
                cdr_profile.url,
                cdr_profile.format
            );
            unsafe { switch_core_add_state_handler(&STATE_HANDLERS) };
            GOLOBAS.write().unwrap().running = true;
        }
    }
}

pub fn shutdown() {
    if GOLOBAS.read().unwrap().running {
        debug!("remove cdr report state handler");
        unsafe { switch_core_remove_state_handler(&STATE_HANDLERS) };
    }
}

pub fn load_config(cfg: switch_xml_t) {
    unsafe {
        let tmp_str = CString::new("cdrs").unwrap();
        let cdrs_tag = switch_xml_child(cfg, tmp_str.as_ptr());
        if cdrs_tag.is_null() {
            warn!("Missing <cdrs> tag!");
            return;
        }

        let mut cdr_profile = Profile::new();
        let tmp_str = CString::new("cdr").unwrap();
        let mut cdr_tag = fsr::switch_xml_child(cdrs_tag, tmp_str.as_ptr());
        while !cdr_tag.is_null() {
            let tmp_str = CString::new("name").unwrap();
            let bname = switch_xml_attr_soft(cdr_tag, tmp_str.as_ptr());
            cdr_profile.name = to_string(bname);

            let tmp_str = CString::new("param").unwrap();
            let mut param = fsr::switch_xml_child(cdr_tag, tmp_str.as_ptr());
            while !param.is_null() {
                let tmp_str = CString::new("name").unwrap();
                let var = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());
                let tmp_str = CString::new("value").unwrap();
                let val = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());

                let var = fsr::to_string(var);
                let val = fsr::to_string(val);

                if var.eq_ignore_ascii_case("url") {
                    cdr_profile.url = val;
                } else if var.eq_ignore_ascii_case("format") {
                    cdr_profile.format = val;
                } else if var.eq_ignore_ascii_case("retries") {
                    cdr_profile.retries = val.parse::<i32>().unwrap_or(0);
                    if cdr_profile.retries < 1 {
                        cdr_profile.retries = 1;
                    }
                    if cdr_profile.retries > 3 {
                        cdr_profile.retries = 3;
                    }
                } else if var.eq_ignore_ascii_case("delay") {
                    cdr_profile.delay = val.parse::<i32>().unwrap_or(20);
                    if cdr_profile.delay < 10 {
                        cdr_profile.delay = 10;
                    }
                    if cdr_profile.delay > 6000 {
                        cdr_profile.delay = 6000;
                    }
                } else if var.eq_ignore_ascii_case("log-http-and-disk") {
                    cdr_profile.log_http_and_disk = fsr::switch_true(&val);
                } else if var.eq_ignore_ascii_case("log-dir") {
                    if val.is_empty() {
                        cdr_profile.log_dir = format!("{}/zrs_cdr", get_variable("log_dir"));
                    } else {
                        cdr_profile.log_dir = val;
                    }
                } else if var.eq_ignore_ascii_case("log-b-leg") {
                    cdr_profile.log_b_leg = fsr::switch_true(&val);
                } else if var.eq_ignore_ascii_case("prefix-a-leg") {
                    cdr_profile.prefix_a_leg = fsr::switch_true(&val);
                } else if var.eq_ignore_ascii_case("err-log-dir") {
                    if val.is_empty() {
                        cdr_profile.err_log_dir = format!("{}/zrs_cdr", get_variable("log_dir"));
                    } else {
                        cdr_profile.err_log_dir = val;
                    }
                } else if var.eq_ignore_ascii_case("timeout") {
                    cdr_profile.timeout = val.parse::<u64>().unwrap_or(20);
                    if cdr_profile.timeout < 10 {
                        cdr_profile.timeout = 10;
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

fn generate_cdr(
    profile: &Profile,
    session: *mut switch_core_session_t,
) -> Result<CdrData, switch_status_t> {
    let mut is_b = false;
    let mut a_prefix: &str = "";
    let mut cdr_text = String::new();
    unsafe {
        let channel = switch_core_session_get_channel(session);
        if !channel.is_null() && !switch_channel_get_originator_caller_profile(channel).is_null() {
            is_b = true;
        }

        if !profile.log_b_leg && is_b {
            let force_cdr = switch_channel_get_variable_dup(
                channel,
                SWITCH_FORCE_PROCESS_CDR_VARIABLE.as_ptr() as *const c_char,
                switch_bool_t::SWITCH_TRUE,
                -1,
            );
            if !switch_true(&to_string(force_cdr)) {
                return Err(switch_status_t::SWITCH_STATUS_SUCCESS);
            }
        }
        if is_b && profile.prefix_a_leg {
            a_prefix = "a_";
        }

        if profile.format.eq_ignore_ascii_case("json") {
            let mut json_cdr = std::ptr::null_mut() as *mut cJSON;

            let encode = || {
                if profile.encode_values {
                    switch_bool_t::SWITCH_TRUE
                } else {
                    switch_bool_t::SWITCH_FALSE
                }
            };

            if switch_ivr_generate_json_cdr(
                session,
                (&mut json_cdr) as *mut _ as *mut *mut cJSON,
                encode(),
            ) != switch_status_t::SWITCH_STATUS_SUCCESS
            {
                error!("Error Generating JSON Data!");
                return Err(switch_status_t::SWITCH_STATUS_FALSE);
            }

            if !json_cdr.is_null() {
                let str = CString::new("is_bleg").unwrap();
                if is_b {
                    cJSON_AddItemToObject(json_cdr, str.as_ptr(), cJSON_CreateBool(1))
                } else {
                    cJSON_AddItemToObject(json_cdr, str.as_ptr(), cJSON_CreateBool(0))
                }

                /* build the JSON */
                let cdr_text_ptr = cJSON_PrintUnformatted(json_cdr);
                cJSON_Delete(json_cdr);

                if cdr_text_ptr.is_null() {
                    error!("Memory Error generating JSON!");
                }
                cdr_text = to_string(cdr_text_ptr);
                fsr::switch_safe_free(cdr_text_ptr as *mut c_void);
            }
        } else {
            let mut xml_cdr = std::ptr::null_mut() as *mut switch_xml;
            if switch_ivr_generate_xml_cdr(
                session,
                (&mut xml_cdr) as *mut _ as *mut *mut switch_xml,
            ) != switch_status_t::SWITCH_STATUS_SUCCESS
            {
                error!("Error Generating XML Data!");
                return Err(switch_status_t::SWITCH_STATUS_FALSE);
            }

            if !xml_cdr.is_null() {
                let var = CString::new("is_bleg").unwrap();
                if is_b {
                    let val = CString::new("true").unwrap();
                    switch_xml_set_attr_d(xml_cdr, var.as_ptr(), val.as_ptr());
                } else {
                    let val = CString::new("false").unwrap();
                    switch_xml_set_attr_d(xml_cdr, var.as_ptr(), val.as_ptr());
                }

                /* build the XML */
                let cdr_text_ptr = switch_xml_toxml_ex(
                    xml_cdr,
                    switch_bool_t::SWITCH_FALSE,
                    switch_bool_t::SWITCH_FALSE,
                );
                switch_xml_free(xml_cdr);

                if cdr_text_ptr.is_null() {
                    error!("Memory Error generating JSON!");
                }

                cdr_text = to_string(cdr_text_ptr);
                fsr::switch_safe_free(cdr_text_ptr as *mut c_void);
            }
        }
    }

    let uuid = unsafe { to_string(switch_core_session_get_uuid(session)) };
    let filename = format!("{}{}.cdr.{}", a_prefix, uuid, profile.format.clone());
    let cdr_data = CdrData {
        fromat: profile.format.clone(),
        filename,
        uuid,
        text: cdr_text,
    };

    Ok(cdr_data)
}

fn process_cdr(profile: Profile, cdr_data: CdrData) {
    let url = profile.url.clone();
    let mut success = false;

    if profile.log_http_and_disk {
        let path = Path::new(&profile.log_dir);
        let now = chrono::Local::now();
        let path = path.join(now.format("%Y").to_string());
        let path = path.join(now.format("%m%d").to_string());
        let mut ok = false;
        if !path.exists() {
            let ret = fs::create_dir_all(path.clone());
            match ret {
                Ok(()) => {
                    ok = true;
                }
                Err(e) => {
                    error!("Error create all dir {}", e);
                }
            }
        } else {
            ok = false;
        }
        if ok {
            let path = path.join(cdr_data.filename.clone());
            let r = fs::write(path, cdr_data.text.clone());
            match r {
                Ok(_) => {}
                Err(e) => {
                    error!("Error writing {} {}", cdr_data.filename, e);
                }
            }
        }
    }

    for cur_try in 0..profile.retries {
        if cur_try > 0 {
            thread::sleep(Duration::from_millis(profile.delay as u64));
        }
        let mut context = "application/json";
        if cdr_data.fromat.eq_ignore_ascii_case("json") {
            context = "text/xml";
        }

        let response = profile
            .client
            .post(url.clone())
            .header(reqwest::header::CONTENT_TYPE, context)
            .timeout(Duration::from_millis(profile.timeout))
            .body(cdr_data.text.clone())
            .send();
        match response {
            Ok(response) => {
                if !response.status().is_success() {
                    error!(
                        "Got error [{}] posting to web server [{}]",
                        response.status().as_str(),
                        url.clone()
                    );
                    if cur_try < profile.retries {
                        warn!("Retry will be with url [{}]", profile.url);
                    }
                } else {
                    success = true;
                    break;
                }
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    }

    if !success {
        error!(
            "Unable to post cdr to web server [{}]",
            cdr_data.uuid.clone()
        );

        if profile.log_errors_to_disk {
            let path = Path::new(&profile.err_log_dir);
            let now = chrono::Local::now();
            let path = path.join(now.format("%Y").to_string());
            let path = path.join(now.format("%m%d").to_string());
            let mut ok = false;
            if !path.exists() {
                let ret = fs::create_dir_all(path.clone());
                match ret {
                    Ok(()) => ok = true,
                    Err(e) => {
                        error!("Error create all dir {}", e);
                    }
                }
            } else {
                ok = true
            }

            if ok {
                let path = path.join(cdr_data.filename.clone());
                let r = fs::write(path, cdr_data.text);
                match r {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error writing {} {}", cdr_data.filename, e);
                    }
                }
            }
        }
    }
}

unsafe extern "C" fn on_reporting(session: *mut switch_core_session_t) -> switch_status_t {
    let profile = GOLOBAS.read().unwrap().profile.clone();
    match profile {
        None => switch_status_t::SWITCH_STATUS_SUCCESS,
        Some(profile) => {
            let cdr = generate_cdr(&profile, session);
            match cdr {
                Ok(cdr) => {
                    process_cdr(profile, cdr);
                    switch_status_t::SWITCH_STATUS_SUCCESS
                }
                Err(status) => status,
            }
        }
    }
}
