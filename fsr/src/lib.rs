#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![allow(improper_ctypes)]

use std::assert;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;

include!("fs.rs");
include!("logs.rs");
include!("module.rs");
include!("event.rs");

pub fn to_string<'a>(p: *const c_char) -> String {
    if p.is_null() {
        return String::from("");
    }
    let cstr = unsafe { std::ffi::CStr::from_ptr(p) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

pub fn get_variable(s: &str) -> String {
    let tmp_str = CString::new(s).unwrap();
    let val = unsafe { switch_core_get_variable(tmp_str.as_ptr()) };
    to_string(val)
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

        let ret: String = to_string((*stream).data as *const c_char);
        libc::free((*stream).data);
        if status == switch_status_t::SWITCH_STATUS_SUCCESS {
            Ok(ret)
        } else {
            Err(format!("-ERR %s Command not found!{}\n", cmd))
        }
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
        let hostname = to_string(switch_core_get_hostname());
        let section = to_string(section);
        let tag_name = to_string(tag_name);
        let key_name = to_string(key_name);
        let key_value = to_string(key_value);
        let basic_data = format!(
            "hostname={}&section={}&tag_name={}&key_name={}&key_value={}",
            hostname, section, tag_name, key_name, key_value
        );
        let basic_data = CString::new(basic_data).unwrap();
        let data =
            switch_event_build_param_string(params, basic_data.as_ptr(), std::ptr::null_mut());

        let text = (*f)(to_string(data));
        let text = CString::new(text).unwrap();
        let ptr = text.into_raw();
        let xml = switch_xml_parse_str(ptr, libc::strlen(ptr));
        let text = CString::from_raw(ptr);
        let text = text.to_str().unwrap_or("");
        if xml.is_null() {
            error!(
                "Error Parsing Result! \ndata: [{}] RESPONSE[{}]\n",
                to_string(data),
                text
            );
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
