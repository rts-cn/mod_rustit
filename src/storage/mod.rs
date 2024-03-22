use std::{ffi::CString, sync::Mutex};

use fsr::*;
use lazy_static::lazy_static;
use libc::c_char;

mod cache;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    /// time to keep files when discoverd they were deleted from the http server
    pub file_not_found_expires: i32,
    /// how often to re-check the server to make sure the remote file has not changed
    pub file_cache_ttl: i32,
    /// storage server url
    pub url: String,
    /// cache temp files path
    pub cache_dir: String,
    /// http cache
    cached: Option<cache::Cache>,
}

impl Profile {
    pub fn new() -> Profile {
        Profile {
            cached: None,
            name: "".to_string(),
            file_not_found_expires: 1,
            file_cache_ttl: 1,
            url: "".to_string(),
            cache_dir: "".to_string(),
        }
    }
}

struct Global {
    profiles: Vec<Profile>,
}
impl Global {
    pub fn new() -> Global {
        Global {
            profiles: Vec::new(),
        }
    }
    pub fn get(name: &str) -> Option<Profile> {
        for profile in &GOLOBAS.lock().unwrap().profiles {
            if profile.name.eq_ignore_ascii_case(name) {
                return Some(profile.clone());
            }
        }
        None
    }
}

lazy_static! {
    static ref GOLOBAS: Mutex<Global> = Mutex::new(Global::new());
}

#[derive(Debug, Clone)]
struct FileContext {
    pub file_url: String,
    pub file_path: String,
    pub stream: String,
    pub cache_file: String,
    pub samples: u32,
    fh: switch_file_handle_t,
    cached: Option<cache::Cache>,
}

impl FileContext {
    pub fn new() -> FileContext {
        FileContext {
            file_url: "".to_string(),
            stream: "".to_string(),
            file_path: "".to_string(),
            cache_file: "".to_string(),
            samples: 10,
            fh: Default::default(),
            cached: None,
        }
    }
    pub fn file_ptr(&mut self) -> *mut switch_file_handle_t {
        (&mut self.fh) as *mut _ as *mut switch_file_handle_t
    }
}

pub fn load_config(cfg: switch_xml_t) {
    lazy_static::initialize(&GOLOBAS);
    unsafe {
        let tmp_str = CString::new("storages").unwrap();
        let storages_tag = switch_xml_child(cfg, tmp_str.as_ptr());
        if storages_tag.is_null() {
            warn!("Missing <storages> tag!");
            return;
        }

        let tmp_str = CString::new("storage").unwrap();
        let mut storage_tag = fsr::switch_xml_child(storages_tag, tmp_str.as_ptr());
        while !storage_tag.is_null() {
            let tmp_str = CString::new("name").unwrap();
            let bname = switch_xml_attr_soft(storage_tag, tmp_str.as_ptr());
            let name = to_string(bname);
            if !name.eq_ignore_ascii_case("hfs") && !name.eq_ignore_ascii_case("s3") {
                storage_tag = (*storage_tag).next;
                continue;
            }
            let mut profile = Profile::new();
            profile.name = name;
            let tmp_str = CString::new("param").unwrap();
            let mut param = fsr::switch_xml_child(storage_tag, tmp_str.as_ptr());
            while !param.is_null() {
                let tmp_str = CString::new("name").unwrap();
                let var = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());
                let tmp_str = CString::new("value").unwrap();
                let val = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());

                let var = fsr::to_string(var);
                let val = fsr::to_string(val);

                if var.eq_ignore_ascii_case("url") {
                    profile.url = val;
                } else if var.eq_ignore_ascii_case("file-not-found-expires") {
                    profile.file_not_found_expires = val.parse::<i32>().unwrap_or(5);
                    if profile.file_not_found_expires < 1 {
                        profile.file_not_found_expires = 1;
                    }
                    if profile.file_not_found_expires > 120 {
                        profile.file_not_found_expires = 120;
                    }
                } else if var.eq_ignore_ascii_case("file-cache-ttl") {
                    profile.file_cache_ttl = val.parse::<i32>().unwrap_or(5);
                    if profile.file_cache_ttl < 1 {
                        profile.file_cache_ttl = 1;
                    }
                    if profile.file_cache_ttl > 120 {
                        profile.file_cache_ttl = 120;
                    }
                } else if var.eq_ignore_ascii_case("cache-dir") {
                    if !val.is_empty() {
                        profile.cache_dir = val;
                    }
                }

                if profile.cache_dir.is_empty() {
                    let storage_dir = get_variable("storage_dir");
                    let path = std::path::Path::new(&storage_dir);
                    let path = path.join(format!("{}_cache", profile.name));
                    profile.cache_dir = path.to_str().unwrap_or_default().to_string();
                }

                param = (*param).next;
            }

            if profile.url.starts_with("http://") || profile.url.starts_with("https://") {
                let cached = cache::Cache::new(&profile.cache_dir);
                match cached {
                    Ok(cached) => {
                        profile.cached = Some(cached);
                        GOLOBAS.lock().unwrap().profiles.push(profile);
                    }
                    Err(e) => {
                        error!("Failed to create cache {}", e);
                    }
                }
            }
            storage_tag = (*storage_tag).next;
        }
    }
}

unsafe extern "C" fn vfs_file_open(
    handle: *mut switch_file_handle_t,
    file_path: *const ::std::os::raw::c_char,
) -> switch_status_t {
    let stream_name = to_string((*handle).stream_name);
    let file_path = to_string(file_path);
    let profile = Global::get(&stream_name);
    match profile {
        None => {
            return switch_status_t::SWITCH_STATUS_FALSE;
        }
        Some(profile) => {
            let mut context = Box::new(FileContext::new());
            context.file_url = format!("{}/{}", profile.url, file_path);
            let fh = context.file_ptr();
            if ((*handle).flags & switch_file_flag_enum_t::SWITCH_FILE_FLAG_WRITE.0) != 0 {
                if let Some(cached) = &profile.cached {
                    context.cache_file = cached.create_cached_file(&file_path);
                }
                (*fh).channels = (*handle).channels;
                (*fh).native_rate = (*handle).native_rate;
                (*fh).samples = (*handle).samples;
                (*fh).samplerate = (*handle).samplerate;
                (*fh).prefix = (*handle).prefix;
            } else {
                if let Some(cached) = &profile.cached {
                    context.cache_file = cached.get(&context.file_url);
                }
            }

            if context.cache_file.is_empty() {
                return switch_status_t::SWITCH_STATUS_FALSE;
            }

            context.file_path = file_path;
            context.stream = stream_name;
            let cache_file = CString::new(context.cache_file.clone()).unwrap();
            let status = switch_core_perform_file_open(
                concat!(file!(), '\0').as_ptr() as *const c_char,
                std::ptr::null_mut(),
                line!() as libc::c_int,
                fh,
                cache_file.as_ptr(),
                (*handle).channels,
                (*handle).samplerate,
                (*handle).flags,
                std::ptr::null_mut(),
            );

            if status != switch_status_t::SWITCH_STATUS_SUCCESS {
                error!("Invalid cache file {}.", context.cache_file);
                return status;
            }

            if ((*fh).flags & switch_file_flag_enum_t::SWITCH_FILE_FLAG_VIDEO.0) != 0 {
                (*handle).flags |= switch_file_flag_enum_t::SWITCH_FILE_FLAG_VIDEO.0;
            } else {
                (*handle).flags &= !switch_file_flag_enum_t::SWITCH_FILE_FLAG_VIDEO.0;
            }

            context.cached = profile.cached;
            (*handle).private_info = Box::leak(context) as *mut _ as *mut std::ffi::c_void;
            (*handle).samples = (*fh).samples;
            (*handle).format = (*fh).format;
            (*handle).sections = (*fh).sections;
            (*handle).seekable = (*fh).seekable;
            (*handle).speed = (*fh).speed;
            (*handle).interval = (*fh).interval;
            (*handle).channels = (*fh).channels;
            (*handle).cur_channels = (*fh).cur_channels;
            (*handle).flags |= switch_file_flag_enum_t::SWITCH_FILE_NOMUX.0;

            if ((*fh).flags & switch_file_flag_enum_t::SWITCH_FILE_NATIVE.0) != 0 {
                (*handle).flags |= switch_file_flag_enum_t::SWITCH_FILE_NATIVE.0;
            } else {
                (*handle).flags &= !switch_file_flag_enum_t::SWITCH_FILE_NATIVE.0;
            }
            status
        }
    }
}

unsafe extern "C" fn vfs_file_close(handle: *mut switch_file_handle_t) -> switch_status_t {
    let mut context = Box::from_raw((*handle).private_info as *mut FileContext);
    let fh = context.file_ptr();
    if ((*fh).flags & switch_file_flag_enum_t::SWITCH_FILE_OPEN.0) != 0 {
        switch_core_file_close(fh);
    }

    if ((*handle).flags & switch_file_flag_enum_t::SWITCH_FILE_FLAG_WRITE.0) != 0 {
        if let Some(cached) = &context.cached {
            let resonse = cached.close_cached_file(&context.file_url, &context.cache_file);
            match resonse {
                Ok(()) => (),
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
    }
    switch_status_t::SWITCH_STATUS_SUCCESS
}

unsafe extern "C" fn vfs_file_read(
    handle: *mut switch_file_handle_t,
    data: *mut ::std::os::raw::c_void,
    len: *mut switch_size_t,
) -> switch_status_t {
    let context = (*handle).private_info as *mut FileContext;
    let mut status = switch_status_t::SWITCH_STATUS_SUCCESS;
    if (*context).samples > 0 {
        if *len > (*context).samples as switch_size_t {
            *len = (*context).samples as switch_size_t;
        }
        (*context).samples -= *len as u32;
        data.write_bytes(255, *len * 2);
    } else {
        status = switch_core_file_read((*context).file_ptr(), data, len);
    }
    status
}

unsafe extern "C" fn vfs_file_write(
    handle: *mut switch_file_handle_t,
    data: *mut ::std::os::raw::c_void,
    len: *mut switch_size_t,
) -> switch_status_t {
    let context = (*handle).private_info as *mut FileContext;
    let status = switch_core_file_write((*context).file_ptr(), data, len);
    status
}

unsafe extern "C" fn vfs_file_seek(
    handle: *mut switch_file_handle_t,
    cur_sample: *mut ::std::os::raw::c_uint,
    samples: i64,
    whence: ::std::os::raw::c_int,
) -> switch_status_t {
    let context = (*handle).private_info as *mut FileContext;
    if (*handle).seekable == 1 {
        warn!("File is not seekable\n");
        return switch_status_t::SWITCH_STATUS_NOTIMPL;
    }
    let fh = (*context).file_ptr();
    let status = switch_core_file_seek(fh, cur_sample, samples, whence);
    if status == switch_status_t::SWITCH_STATUS_SUCCESS {
        (*handle).pos = (*fh).pos;
        (*handle).offset_pos = (*fh).offset_pos;
        (*handle).samples_in = (*fh).samples_in;
        (*handle).samples_out = (*fh).samples_out;
    }
    status
}

unsafe extern "C" fn vfs_file_read_video(
    handle: *mut switch_file_handle_t,
    frame: *mut switch_frame_t,
    flags: switch_video_read_flag_t,
) -> switch_status_t {
    let context = (*handle).private_info as *mut FileContext;
    switch_core_file_read_video((*context).file_ptr(), frame, flags)
}

unsafe extern "C" fn vfs_file_write_video(
    handle: *mut switch_file_handle_t,
    frame: *mut switch_frame_t,
) -> switch_status_t {
    let context = (*handle).private_info as *mut FileContext;
    switch_core_file_write_video((*context).file_ptr(), frame)
}

pub fn shutdown() {
    loop {
        let profile = GOLOBAS.lock().unwrap().profiles.pop();
        match profile {
            Some(profile) => {
                if let Some(cached) = profile.cached {
                    cached.close();
                }
            }
            None => {
                break;
            }
        }
    }
}

pub fn start(m: &fsr::Module, name: &str) {
    let profiles = &GOLOBAS.lock().unwrap().profiles;

    for profile in profiles {
        notice!(
            "Binding [{}] File Interface [{}]",
            profile.name,
            profile.url
        );

        unsafe {
            let extens = switch_alloc!(
                m.pool(),
                std::mem::size_of::<*const c_char>() * SWITCH_MAX_CODECS as usize
            ) as *mut *mut c_char;

            (*extens) = strdup!(m.pool(), &profile.name);

            let fi: *mut switch_file_interface_t = m
                .create_interface(switch_module_interface_name_t::SWITCH_FILE_INTERFACE)
                as *mut switch_file_interface_t;
            (*fi).extens = extens;
            (*fi).interface_name = strdup!(m.pool(), name);
            (*fi).file_open = Some(vfs_file_open);
            (*fi).file_close = Some(vfs_file_close);
            (*fi).file_read = Some(vfs_file_read);
            (*fi).file_write = Some(vfs_file_write);
            (*fi).file_seek = Some(vfs_file_seek);
            (*fi).file_read_video = Some(vfs_file_read_video);
            (*fi).file_write_video = Some(vfs_file_write_video);
        }
    }
}
