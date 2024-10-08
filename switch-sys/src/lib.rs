#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![allow(improper_ctypes)]

use std::assert;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;

include!("base.rs");
include!("utils.rs");
include!("logs.rs");
include!("module.rs");
include!("event.rs");

pub fn get_variable(s: &str) -> String {
    let tmp_str = CString::new(s).unwrap();
    let val = unsafe { switch_core_get_variable(tmp_str.as_ptr()) };
    switch_to_string(val)
}

pub fn set_variable(name: &str, val: &str) {
    let name = CString::new(name).unwrap();
    let val = CString::new(val).unwrap();
    unsafe { switch_core_set_variable(name.as_ptr(), val.as_ptr()) };
}

pub fn check_acl(ip: &str, list: &str) -> bool {
    let cstr_ip = CString::new(ip).unwrap();
    let cstr_list = CString::new(list).unwrap();
    let mut token = std::ptr::null() as *const c_char;

    let r = unsafe {
        switch_check_network_list_ip_token(
            cstr_ip.as_ptr(),
            cstr_list.as_ptr(),
            std::ptr::addr_of_mut!(token),
        )
    };
    if r == switch_bool_t::SWITCH_TRUE {
        true
    } else {
        false
    }
}

/// api_exec
/// execute FreeSWITCH api
pub fn api_exec(cmd: &str, arg: &str) -> Result<String, String> {
    unsafe {
        let data_size: usize = 1024;
        let data = libc::malloc(data_size);
        libc::memset(data, 0, data_size);
        let stream = &mut switch_stream_handle_t {
            read_function: None,
            write_function: Some(switch_console_stream_write),
            raw_write_function: Some(switch_console_stream_raw_write),
            data,
            data_size,
            data_len: 0,
            alloc_len: data_size,
            alloc_chunk: data_size,
            param_event: std::ptr::null_mut() as *mut switch_event,
            end: data,
        };

        let api_cmd = CString::new(cmd).unwrap();
        let api_arg = CString::new(arg).unwrap();

        let status = switch_api_execute(
            api_cmd.as_ptr(),
            api_arg.as_ptr(),
            std::ptr::null_mut() as *mut switch_core_session_t,
            stream as *mut switch_stream_handle_t,
        );

        let ret: String = switch_to_string((*stream).data as *const c_char);
        switch_safe_free((*stream).data);
        if status == switch_status_t::SWITCH_STATUS_SUCCESS {
            Ok(ret)
        } else {
            Err(format!("-ERR %s Command not found!{}\n", cmd))
        }
    }
}

/// json_api_exec
/// execute FreeSWITCH json api
pub fn json_api_exec(cmd: &str) -> Result<String, String> {
    unsafe {
        let parse_error =
            String::from(r#"{"status":"error","message":"Parse command error","response":null}"#);
        if !cmd.starts_with("{") || !cmd.ends_with("}") {
            return Err(parse_error);
        }
        let jcmd = CString::new(cmd).unwrap();
        let jcmd = cJSON_Parse(jcmd.as_ptr());
        if jcmd.is_null() {
            return Err(parse_error);
        }

        switch_json_api_execute(
            jcmd,
            std::ptr::null_mut() as *mut switch_core_session_t,
            std::ptr::null_mut() as *mut *mut cJSON,
        );

        let key: CString = CString::new("command").unwrap();
        cJSON_DeleteItemFromObject(jcmd, key.as_ptr());
        let key: CString = CString::new("data").unwrap();
        cJSON_DeleteItemFromObject(jcmd, key.as_ptr());

        let json_text = cJSON_PrintUnformatted(jcmd);
        let response = switch_to_string(json_text);
        switch_safe_free(json_text as *mut c_void);
        cJSON_Delete(jcmd);
        Ok(response)
    }
}

/// sendevent
/// send event to FreeSWITCH
pub fn sendevent<'a>(
    id: u32,
    subclass_name: &'a str,
    header: HashMap<String, String>,
    body: &'a str,
) -> Result<String, String> {
    let mut event = std::ptr::null_mut() as *mut switch_event_t;
    unsafe {
        let mut sub_name = std::ptr::null() as *const c_char;
        if id == switch_event_types_t::SWITCH_EVENT_CUSTOM.0 && !subclass_name.is_empty() {
            let subclass = CString::new(subclass_name).unwrap();
            sub_name = subclass.as_ptr()
        }
        let status = switch_event_create_subclass_detailed(
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            std::ptr::null(),
            line!() as std::os::raw::c_int,
            (&mut event) as *mut _ as *mut *mut switch_event_t,
            switch_event_types_t(id),
            sub_name,
        );

        if status != switch_status_t::SWITCH_STATUS_SUCCESS {
            return Err(String::from("-ERR create event error"));
        }

        let mut uuid_str: [c_char; 256] = [0; 256];
        switch_uuid_str(uuid_str.as_mut_ptr(), uuid_str.len());
        let header_name = CString::new("Event-UUID").unwrap();
        switch_event_add_header_string(
            event,
            switch_stack_t::SWITCH_STACK_BOTTOM,
            header_name.as_ptr(),
            uuid_str.as_ptr(),
        );

        for (key, value) in header {
            let name = CString::new(key).unwrap();
            let data = CString::new(value).unwrap();
            switch_event_add_header_string(
                event,
                switch_stack_t::SWITCH_STACK_BOTTOM,
                name.as_ptr(),
                data.as_ptr(),
            );
        }

        let c_body = CString::new(body).unwrap();
        switch_event_set_body(event, c_body.as_ptr());

        switch_event_fire_detailed(
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            std::ptr::null(),
            line!() as std::os::raw::c_int,
            (&mut event) as *mut _ as *mut *mut switch_event_t,
            std::ptr::null_mut(),
        )
    };
    Ok(String::from("+OK"))
}

/// sendmsg
/// send msg to FreeSWITCH
pub fn sendmsg<'a>(uuid: &'a str, header: HashMap<String, String>) -> Result<String, String> {
    if uuid.is_empty() {
        return Err(String::from("-ERR invalid session id"));
    }

    let mut event = std::ptr::null_mut() as *mut switch_event_t;
    unsafe {
        let status = switch_event_create_subclass_detailed(
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            std::ptr::null(),
            line!() as std::os::raw::c_int,
            (&mut event) as *mut _ as *mut *mut switch_event_t,
            switch_event_types_t::SWITCH_EVENT_CLONE,
            std::ptr::null(),
        );

        if status != switch_status_t::SWITCH_STATUS_SUCCESS {
            return Err(String::from("create event error"));
        }

        for (key, value) in header {
            let name = CString::new(key).unwrap();
            let data = CString::new(value).unwrap();
            switch_event_add_header_string(
                event,
                switch_stack_t::SWITCH_STACK_BOTTOM,
                name.as_ptr(),
                data.as_ptr(),
            );
        }

        let uuid_str = CString::new(uuid).unwrap();
        let session = switch_core_session_perform_locate(
            uuid_str.as_ptr(),
            concat!(file!(), '\0').as_ptr() as *const std::os::raw::c_char,
            std::ptr::null(),
            line!() as std::os::raw::c_int,
        );

        if !session.is_null() {
            let status = switch_core_session_queue_private_event(
                session,
                (&mut event) as *mut _ as *mut *mut switch_event_t,
                switch_bool_t::SWITCH_FALSE,
            );

            switch_core_session_rwunlock(session);
            if status != switch_status_t::SWITCH_STATUS_SUCCESS {
                return Err(String::from("-ERR memory error"));
            }
        } else {
            return Err(String::from(format!("-ERR invalid session id [{}]", uuid)));
        }
    };

    Ok(String::from("+OK"))
}

/// xml_bind_search
/// Add FreeSWITCH XMLBinding
///
/// This macro will add a FreeSWICH XMLBinding
/// # Examples
///
/// ```
/// fn example(data:String) -> String
/// {
///    todo!()
/// }
/// xml_bind_search("configuration|directory|dialplan", example);
/// ```
pub fn xml_bind_search<F>(bindings: &str, callback: F) -> u64
where
    F: Fn(String) -> String,
{
    unsafe extern "C" fn wrap_callback<F>(
        section: *const c_char,
        tag_name: *const c_char,
        key_name: *const c_char,
        key_value: *const c_char,
        params: *mut switch_event_t,
        user_data: *mut c_void,
    ) -> switch_xml_t
    where
        F: Fn(String) -> String,
    {
        let f = user_data as *mut F;
        let fmt =
            CString::new("hostname=%s&section=%s&tag_name=%s&key_name=%s&key_value=%s").unwrap();
        let basic_data = switch_mprintf(
            fmt.as_ptr(),
            switch_core_get_hostname(),
            section,
            tag_name,
            key_name,
            key_value,
        );

        let params_str = switch_event_build_param_string(params, basic_data, std::ptr::null_mut());
        let data = switch_to_string(params_str);
        switch_safe_free(basic_data as *mut c_void);
        switch_safe_free(params_str as *mut c_void);

        let response = (*f)(data);
        let response = CString::new(response).unwrap();
        let xml = switch_xml_parse_str_dynamic(
            response.as_ptr() as *mut c_char,
            switch_bool_t::SWITCH_TRUE,
        );
        if xml.is_null() {
            warn!("Error Parsing XML:\n{:?}", response);
        }
        xml
    }
    unsafe {
        let bindings = CString::new(bindings).unwrap();
        let sections = switch_xml_parse_section_string(bindings.as_ptr());

        let mut ret_binding = 0 as *mut u64;
        let fp = std::ptr::addr_of!(callback);
        switch_xml_bind_search_function_ret(
            Some(wrap_callback::<F>),
            sections,
            fp as *mut c_void,
            (&mut ret_binding) as *mut _ as *mut *mut switch_xml_binding,
        );
        ret_binding as u64
    }
}

pub fn xml_unbind_search(binding: u64) {
    let mut binding = binding as *mut u64;
    unsafe {
        switch_xml_unbind_search_function((&mut binding) as *mut _ as *mut *mut switch_xml_binding);
    }
}

pub struct SytemStatus {
    pub uptime: i64,
    pub version: String,
    pub ready: bool,
    pub session_total: u64,
    pub session_active: u32,
    pub session_peak: i32,
    pub session_peak_5min: i32,
    pub session_limit: u32,
    pub rate_current: i32,
    pub rate_max: i32,
    pub rate_peak: i32,
    pub rate_peak_5min: i32,
    pub idle_cpu_used: f64,
    pub idle_cpu_allowed: f64,
    pub stack_size_current: f32,
    pub stack_size_max: f32,
}

impl Default for SytemStatus {
    fn default() -> SytemStatus {
        SytemStatus {
            uptime: 0,
            version: String::new(),
            ready: false,
            session_total: 0,
            session_active: 0,
            session_peak: 0,
            session_peak_5min: 0,
            session_limit: 0,
            rate_current: 0,
            rate_max: 0,
            rate_peak: 0,
            rate_peak_5min: 0,
            idle_cpu_allowed: 0.0,
            idle_cpu_used: 0.0,
            stack_size_current: 0.0,
            stack_size_max: 0.0,
        }
    }
}

pub fn status() -> SytemStatus {
    unsafe {
        let mut status = SytemStatus::default();

        let mut sps = 0;
        let mut last_sps = 0;
        let mut max_sps = 0;
        let mut max_sps_fivemin = 0;
        let mut sessions_peak = 0;
        let mut sessions_peak_fivemin = 0;
        let mut cur: switch_size_t = 0;
        let mut max: switch_size_t = 0;

        switch_core_session_ctl(
            switch_session_ctl_t::SCSC_SESSIONS_PEAK,
            (&mut sessions_peak) as *mut _ as *mut c_void,
        );
        switch_core_session_ctl(
            switch_session_ctl_t::SCSC_SESSIONS_PEAK_FIVEMIN,
            (&mut sessions_peak_fivemin) as *mut _ as *mut c_void,
        );
        switch_core_session_ctl(
            switch_session_ctl_t::SCSC_LAST_SPS,
            (&mut last_sps) as *mut _ as *mut c_void,
        );
        switch_core_session_ctl(
            switch_session_ctl_t::SCSC_SPS,
            (&mut sps) as *mut _ as *mut c_void,
        );
        switch_core_session_ctl(
            switch_session_ctl_t::SCSC_SPS_PEAK,
            (&mut max_sps) as *mut _ as *mut c_void,
        );
        switch_core_session_ctl(
            switch_session_ctl_t::SCSC_SPS_PEAK_FIVEMIN,
            (&mut max_sps_fivemin) as *mut _ as *mut c_void,
        );

        if switch_core_ready() == switch_bool_t::SWITCH_TRUE {
            status.ready = true;
        }
        status.uptime = switch_core_uptime();
        status.version = switch_to_string(switch_version_full());

        status.session_total = (switch_core_session_id() - 1) as u64;
        status.session_active = switch_core_session_count();
        status.session_peak = sessions_peak;
        status.session_peak_5min = sessions_peak_fivemin;
        status.session_limit = switch_core_session_limit(0);

        status.rate_current = last_sps;
        status.rate_max = sps;
        status.rate_peak = max_sps;
        status.rate_peak_5min = max_sps_fivemin;

        status.idle_cpu_used = switch_core_min_idle_cpu(-1.0);
        status.idle_cpu_allowed = switch_core_idle_cpu();

        if switch_core_get_stacksizes(
            (&mut cur) as *mut _ as *mut usize,
            (&mut max) as *mut _ as *mut usize,
        ) == switch_status_t::SWITCH_STATUS_SUCCESS
        {
            status.stack_size_current = (cur / 1024) as f32;
            status.stack_size_max = (max / 1024) as f32;
        }
        status
    }
}

///  Evaluate the truthfullness of a string expression
///  param expr a string expression
///  return true or false
pub fn switch_true(str: &str) -> bool {
    if str.eq_ignore_ascii_case("yes")
        || str.eq_ignore_ascii_case("on")
        || str.eq_ignore_ascii_case("true")
        || str.eq_ignore_ascii_case("t")
        || str.eq_ignore_ascii_case("enabled")
        || str.eq_ignore_ascii_case("active")
        || str.eq_ignore_ascii_case("allow")
    {
        return true;
    }
    let num = str.parse::<i32>();
    match num {
        Ok(n) => {
            if n > 0 {
                return true;
            }
        }
        Err(_) => (),
    }
    false
}
