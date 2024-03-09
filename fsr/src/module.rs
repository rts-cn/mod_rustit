pub struct Session(*mut switch_core_session_t);
impl Session {
    pub unsafe fn from_ptr(p: *mut switch_core_session_t) -> Session {
        Session(p)
    }
    pub fn as_ptr(&self) -> *const switch_core_session_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut switch_core_session_t {
        self.0
    }
}

pub struct Stream(*mut switch_stream_handle_t);
impl Stream {
    pub unsafe fn from_ptr(p: *mut switch_stream_handle_t) -> Stream {
        assert!(!p.is_null());
        Stream(p)
    }
    pub fn as_ptr(&self) -> *const switch_stream_handle_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut switch_stream_handle_t {
        self.0
    }
    pub fn write(&self, s: &str) {
        let ok = CString::new(s).expect("CString::new failed");
        unsafe {
            (*self.0).write_function.unwrap()(self.0, ok.as_ptr());
        }
    }
}

pub struct Module {
    module: *mut switch_loadable_module_interface_t,
    pool: *mut switch_memory_pool_t,
}

impl Module {
    pub unsafe fn from_ptr(
        module: *mut switch_loadable_module_interface_t,
        pool: *mut switch_memory_pool_t,
    ) -> Module {
        assert!(!pool.is_null());
        assert!(!module.is_null());
        Module { module, pool }
    }
    pub fn as_ptr(&mut self) -> *const switch_loadable_module_interface_t {
        self.module
    }
    pub fn as_mut_ptr(&mut self) -> *mut switch_loadable_module_interface_t {
        self.module
    }
    pub unsafe fn as_ref(&self) -> &switch_loadable_module_interface_t {
        &*self.module
    }
    pub unsafe fn as_mut_ref(&self) -> &mut switch_loadable_module_interface_t {
        &mut *self.module
    }

    pub unsafe fn pool(&self) -> *mut switch_memory_pool_t {
        self.pool
    }

    pub fn create_interface(&self, iname: switch_module_interface_name_t) -> *mut c_void {
        unsafe { switch_loadable_module_create_interface((*self).module, iname) }
    }

    pub fn add_api(&self, name: &str, desc: &str, syntax: &str, func: switch_api_function_t) {
        unsafe {
            let name = strdup!(self.pool(), name);
            let desc = strdup!(self.pool(), desc);
            let syntax = strdup!(self.pool(), syntax);
            let api = self.create_interface(switch_module_interface_name_t::SWITCH_API_INTERFACE)
                as *mut switch_api_interface_t;
            assert!(!api.is_null());
            (*api).interface_name = name;
            (*api).desc = desc;
            (*api).syntax = syntax;
            (*api).function = func;
        }
    }

    pub fn add_application(
        &self,
        name: &str,
        short_desc: &str,
        long_desc: &str,
        syntax: &str,
        func: switch_application_function_t,
        flags: switch_application_flag_enum_t,
    ) {
        unsafe {
            let name = strdup!(self.pool(), name);
            let long_desc = strdup!(self.pool(), long_desc);
            let short_desc = strdup!(self.pool(), short_desc);
            let syntax = strdup!(self.pool(), syntax);
            let app = self.create_interface(switch_module_interface_name_t::SWITCH_APPLICATION_INTERFACE)
                as *mut switch_application_interface;
            assert!(!app.is_null());
            (*app).interface_name = name;
            (*app).long_desc = long_desc;
            (*app).short_desc = short_desc;
            (*app).syntax = syntax;
            (*app).flags = flags.0;
            (*app).application_function = func;
        }
    }
}

/// Create FreeSWITCH module interface
///
/// # Examples
/// ```
/// use fsr::switch_status_t;
/// fn examples_mod_load(m: &fsr::Module) -> switch_status_t { SWITCH_STATUS_SUCCESS }
/// fn examples_mod_runtime() -> switch_status_t { SWITCH_STATUS_SUCCESS }
/// fn examples_mod_shutdown() -> switch_status_t { SWITCH_STATUS_SUCCESS }
/// fsr_mod!("mod_examples", examples_mod_load, examples_mod_runtime, examples_mod_shutdown);
///
/// ```
#[macro_export]
macro_rules! fsr_mod {
    ($name:expr,$load:expr,$runtime:expr,$shutdown:expr) => {
        paste::paste! {
        #[no_mangle]
        pub unsafe extern "C" fn _mod_load(
            mod_int: *mut *mut switch_loadable_module_interface,
            mem_pool: *mut switch_memory_pool_t,
        ) -> switch_status_t {
            let name = strdup!(mem_pool, $name);
            *mod_int = switch_loadable_module_create_module_interface(mem_pool, name);
            if (*mod_int).is_null() {
                return switch_status_t::SWITCH_STATUS_MEMERR;
            }
            let mi = &Module::from_ptr(*mod_int, mem_pool);
            $load(mi)
        }

        #[no_mangle]
        pub extern "C" fn _mod_runtime() -> switch_status_t {
            $runtime()
        }

        #[no_mangle]
        pub extern "C" fn _mod_shutdown() -> switch_status_t {
            $shutdown()
        }

        #[no_mangle]
        #[allow(non_upper_case_globals)]
        pub static mut [<$name _module_interface>]: switch_loadable_module_function_table =
            switch_loadable_module_function_table {
                switch_api_version: SWITCH_API_VERSION as i32,
                load: Some(_mod_load),
                shutdown: Some(_mod_shutdown),
                runtime: Some(_mod_runtime),
                flags: switch_module_flag_enum_t::SMODF_NONE.0,
            };
        }
    };
}

/// Add FreeSWITCH Appliction
///
/// This macro will add a FreeSWICH application
///
/// # Examples
///
/// ```
/// use fsr::switch_application_flag_enum_t;
/// fn examples(session &fsr::session, cmd String)
/// {
///     info!({}, cmd);
/// }
/// fsr_app!(module_interface, "app_name", "long_desc", "short_desc", "syntax", examples, SAF_NONE);
/// ```
#[macro_export]
macro_rules! fsr_app {
    ($module:expr,$name:expr,$short_desc:expr,$long_desc:expr,$syntax:expr,$callback:ident, $flag:expr) => {
        paste::paste! {
            unsafe extern "C" fn [<_app_wrap_ $callback>](
                session: *mut switch_core_session_t,
                cmd: *const ::std::os::raw::c_char
            ) {
                let session = &fsr::Session::from_ptr(session);
                $callback(session, to_string(cmd));
            }

            $module.add_application($name, $short_desc, $long_desc, $syntax, Some([<_app_wrap_ $callback>]), $flag);
        }
    };
}

/// Add FreeSWITCH API
///
/// This macro will add a FreeSWICH API
/// # Examples
///
/// ```
/// fn examples(session &fsr::Session, cmd String, stream &fsr::Stream) -> fsr::switch_status_t
/// {
///    info!({}, cmd);
//     stream.write("OK");
///    switch_status_t::SWITCH_STATUS_SUCCESS
/// }
/// fsr_api!(module_interface, "api_name", "desc", "syntax", examples);
/// ```
#[macro_export]
macro_rules! fsr_api {
    ($module:expr,$name:expr,$desc:expr,$syntax:expr,$callback:ident) => {
        paste::paste! {
                unsafe extern "C" fn [<_api_wrap_ $callback>](
                    cmd: *const std::os::raw::c_char,
                    session: *mut fsr::switch_core_session,
                    stream: *mut fsr::switch_stream_handle_t,
                ) -> switch_status_t {
                    let session = &fsr::Session::from_ptr(session);
                    let stream = &fsr::Stream::from_ptr(stream);
                    $callback(session, to_string(cmd), stream)
                }
            $module.add_api($name, $desc, $syntax, Some([<_api_wrap_ $callback>]));
        }
    };
}
