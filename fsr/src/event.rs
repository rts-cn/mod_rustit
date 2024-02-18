pub struct Event(*mut switch_event_t);
impl Event {
    pub unsafe fn from_ptr(p: *mut switch_event_t) -> Event {
        assert!(!p.is_null());
        Event(p)
    }
    pub fn as_ptr(&self) -> *const switch_event_t {
        self.0
    }
    pub fn as_mut_ptr(&mut self) -> *mut switch_event_t {
        self.0
    }
    pub unsafe fn as_ref(&self) -> &switch_event_t {
        &*self.0
    }
    pub unsafe fn as_mut_ref(&mut self) -> &mut switch_event_t {
        &mut *self.0
    }
    pub fn event_id(&self) -> u32 {
        unsafe { (*self.0).event_id.0 }
    }
    pub fn priority(&self) -> u32 {
        unsafe { (*self.0).priority.0 }
    }
    pub fn owner(&self) -> String {
        unsafe { self::to_string((*self.0).owner) }
    }
    pub fn subclass_name(&self) -> String {
        unsafe { self::to_string((*self.0).subclass_name) }
    }
    pub fn body(&self) -> String {
        unsafe { self::to_string((*self.0).body) }
    }
    pub fn key(&self) -> u64 {
        unsafe { (*self.0).key as u64 }
    }
    pub fn flags(&self) -> i32 {
        unsafe { (*self.0).flags }
    }
    pub fn header<'a>(&'a self, name: &str) -> String {
        unsafe {
            let hname: CString = CString::new(name).expect("CString::new");
            let v = switch_event_get_header_idx(self.0, hname.as_ptr(), -1);
            self::to_string(v)
        }
    }
    pub fn headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        unsafe {
            let mut hp = { *self.0 }.headers;
            loop {
                if hp.is_null() {
                    break;
                }
                headers.insert(to_string((*hp).name), to_string((*hp).value));
                hp = (*hp).next;
            }
        }
        headers
    }
    pub fn string<'a>(&'a self) -> String {
        unsafe {
            let mut s: *mut c_char = std::ptr::null_mut();
            switch_event_serialize(
                self.0,
                std::ptr::addr_of_mut!(s),
                switch_bool_t::SWITCH_FALSE,
            );
            let text = self::to_string(s);
            libc::free(s as *mut c_void);
            text
        }
    }
    pub fn json<'a>(&'a self) -> String {
        unsafe {
            let mut s: *mut c_char = std::ptr::null_mut();
            switch_event_serialize_json(self.0, std::ptr::addr_of_mut!(s));
            let text = self::to_string(s);
            libc::free(s as *mut c_void);
            text
        }
    }
}

pub fn event_bind<F>(
    m: &Module,
    id: &str,
    event: switch_event_types_t,
    subclass_name: Option<&str>,
    callback: F,
) -> u64
where
    F: Fn(Event),
{
    unsafe extern "C" fn wrap_callback<F>(e: *mut switch_event_t)
    where
        F: Fn(Event),
    {
        assert!(!e.is_null());
        assert!(!((*e).bind_user_data.is_null()));
        let f = (*e).bind_user_data as *const F;
        let e = Event::from_ptr(e);
        (*f)(e);
    }
    let fp = std::ptr::addr_of!(callback);
    unsafe {
        let id = strdup!(m.pool(), id);
        let subclass_name = subclass_name.map_or(std::ptr::null(), |x| strdup!(m.pool(), x));
        let mut enode = 0 as *mut u64;
        switch_event_bind_removable(
            id,
            event,
            subclass_name,
            Some(wrap_callback::<F>),
            fp as *mut c_void,
            (&mut enode) as *mut _ as *mut *mut switch_event_node_t,
        );
        enode as u64
    }
}

pub fn event_unbind(id: u64) {
    let mut enode = id as *mut u64;
    unsafe {
        switch_event_unbind((&mut enode) as *mut _ as *mut *mut switch_event_node_t);
    }
}
