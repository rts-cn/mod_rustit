use std::ffi::CString;

use fsr::*;
use libc::c_void;

#[derive(Debug, Clone)]
pub struct CdrData {
    fromat: String,
    text: String,
    uuid: String,
    filename: String,
}

pub fn generate_cdr(
    profile: &super::Profile,
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
                SWITCH_FORCE_PROCESS_CDR_VARIABLE.as_ptr() as *const std::ffi::c_char,
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
                fsr::switch_safe_free(cdr_text_ptr as *mut std::os::raw::c_void);
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
    let filename = format!("{}{}.cdr.{}", a_prefix, uuid, profile.format);
    let cdr_data = CdrData {
        fromat: profile.format.clone(),
        filename,
        uuid,
        text: cdr_text,
    };

    Ok(cdr_data)
}

pub fn process_cdr(profile: super::Profile, cdr_data: CdrData) {
    let mut success = false;

    if profile.log_http_and_disk {
        let path = std::path::Path::new(&profile.log_dir);
        let now = chrono::Local::now();
        let path = path.join(now.format("%Y").to_string());
        let path = path.join(now.format("%m%d").to_string());
        let mut ok = false;
        if !path.exists() {
            let ret = std::fs::create_dir_all(path.as_path());
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
            let path = path.join(&cdr_data.filename);
            let r = std::fs::write(path, &cdr_data.text);
            match r {
                Ok(_) => {}
                Err(e) => {
                    error!("Error writing {} {}", cdr_data.filename, e);
                }
            }
        }
    }

    for cur_try in 0..=profile.retries {
        if cur_try > 0 {
            std::thread::sleep(std::time::Duration::from_secs(profile.delay as u64));
        }
        let mut context = "application/json";
        if cdr_data.fromat.eq_ignore_ascii_case("json") {
            context = "text/xml";
        }
        let response = profile
            .client
            .post(profile.url.as_str())
            .header(reqwest::header::CONTENT_TYPE, context)
            .timeout(std::time::Duration::from_millis(profile.timeout))
            .body(cdr_data.text.clone())
            .send();
        match response {
            Ok(response) => {
                if !response.status().is_success() {
                    error!(
                        "Got error [{}] posting to web server [{}]",
                        response.status().as_str(),
                        profile.url
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
        error!("Unable to post cdr to web server [{}]", &cdr_data.uuid);
        if profile.log_errors_to_disk {
            let path = std::path::Path::new(&profile.err_log_dir);
            let now = chrono::Local::now();
            let path = path.join(now.format("%Y").to_string());
            let path = path.join(now.format("%m%d").to_string());
            let mut ok = false;
            if !path.exists() {
                let ret = std::fs::create_dir_all(path.as_path());
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
                let path = path.join(&cdr_data.filename);
                let r = std::fs::write(path, cdr_data.text);
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
