use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: u32,
    pub priority: u32,
    pub owner: String,
    pub subclass_name: String,
    pub key: u64,
    pub flags: i32,
    pub headers: ::std::collections::HashMap<String, String>,
    pub body: String,
}

impl Event {
    pub fn from_ptr(p: *mut switch_event_t) -> Event {
        assert!(!p.is_null());
        unsafe {
            let mut headers = HashMap::new();

            let mut hp = (*p).headers;
            loop {
                if hp.is_null() {
                    break;
                }
                headers.insert(switch_to_string((*hp).name), switch_to_string((*hp).value));
                hp = (*hp).next;
            }
            Event {
                event_id: (*p).event_id.0,
                priority: (*p).priority.0,
                owner: switch_to_string((*p).owner),
                subclass_name: switch_to_string((*p).subclass_name),
                key: (*p).key as u64,
                flags: (*p).flags as i32,
                headers,
                body: switch_to_string((*p).body),
            }
        }
    }
    pub fn json(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
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
        let id = switch_core_strdup!(m.pool(), id);
        let subclass_name =
            subclass_name.map_or(std::ptr::null(), |x| switch_core_strdup!(m.pool(), x));
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
impl switch_event_types_t {
    pub fn event_id_str_name(self) -> &'static str {
        match self {
            switch_event_types_t::SWITCH_EVENT_CUSTOM => "SWITCH_EVENT_CUSTOM",
            switch_event_types_t::SWITCH_EVENT_CLONE => "SWITCH_EVENT_CLONE",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_CREATE => "SWITCH_EVENT_CHANNEL_CREATE",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_DESTROY => "SWITCH_EVENT_CHANNEL_DESTROY",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_STATE => "SWITCH_EVENT_CHANNEL_STATE",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_CALLSTATE => {
                "SWITCH_EVENT_CHANNEL_CALLSTATE"
            }
            switch_event_types_t::SWITCH_EVENT_CHANNEL_ANSWER => "SWITCH_EVENT_CHANNEL_ANSWER",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_HANGUP => "SWITCH_EVENT_CHANNEL_HANGUP",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_HANGUP_COMPLETE => {
                "SWITCH_EVENT_CHANNEL_HANGUP_COMPLETE"
            }
            switch_event_types_t::SWITCH_EVENT_CHANNEL_EXECUTE => "SWITCH_EVENT_CHANNEL_EXECUTE",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_EXECUTE_COMPLETE => {
                "SWITCH_EVENT_CHANNEL_EXECUTE_COMPLETE"
            }
            switch_event_types_t::SWITCH_EVENT_CHANNEL_HOLD => "SWITCH_EVENT_CHANNEL_HOLD",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_UNHOLD => "SWITCH_EVENT_CHANNEL_UNHOLD",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_BRIDGE => "SWITCH_EVENT_CHANNEL_BRIDGE",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_UNBRIDGE => "SWITCH_EVENT_CHANNEL_UNBRIDGE",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_PROGRESS => "SWITCH_EVENT_CHANNEL_PROGRESS",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_PROGRESS_MEDIA => {
                "SWITCH_EVENT_CHANNEL_PROGRESS_MEDIA"
            }
            switch_event_types_t::SWITCH_EVENT_CHANNEL_OUTGOING => "SWITCH_EVENT_CHANNEL_OUTGOING",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_PARK => "SWITCH_EVENT_CHANNEL_PARK",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_UNPARK => "SWITCH_EVENT_CHANNEL_UNPARK",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_APPLICATION => {
                "SWITCH_EVENT_CHANNEL_APPLICATION"
            }
            switch_event_types_t::SWITCH_EVENT_CHANNEL_ORIGINATE => {
                "SWITCH_EVENT_CHANNEL_ORIGINATE"
            }
            switch_event_types_t::SWITCH_EVENT_CHANNEL_UUID => "SWITCH_EVENT_CHANNEL_UUID",
            switch_event_types_t::SWITCH_EVENT_API => "SWITCH_EVENT_API",
            switch_event_types_t::SWITCH_EVENT_LOG => "SWITCH_EVENT_LOG",
            switch_event_types_t::SWITCH_EVENT_INBOUND_CHAN => "SWITCH_EVENT_INBOUND_CHAN",
            switch_event_types_t::SWITCH_EVENT_OUTBOUND_CHAN => "SWITCH_EVENT_OUTBOUND_CHAN",
            switch_event_types_t::SWITCH_EVENT_STARTUP => "SWITCH_EVENT_STARTUP",
            switch_event_types_t::SWITCH_EVENT_SHUTDOWN => "SWITCH_EVENT_SHUTDOWN",
            switch_event_types_t::SWITCH_EVENT_PUBLISH => "SWITCH_EVENT_PUBLISH",
            switch_event_types_t::SWITCH_EVENT_UNPUBLISH => "SWITCH_EVENT_UNPUBLISH",
            switch_event_types_t::SWITCH_EVENT_TALK => "SWITCH_EVENT_TALK",
            switch_event_types_t::SWITCH_EVENT_NOTALK => "SWITCH_EVENT_NOTALK",
            switch_event_types_t::SWITCH_EVENT_SESSION_CRASH => "SWITCH_EVENT_SESSION_CRASH",
            switch_event_types_t::SWITCH_EVENT_MODULE_LOAD => "SWITCH_EVENT_MODULE_LOAD",
            switch_event_types_t::SWITCH_EVENT_MODULE_UNLOAD => "SWITCH_EVENT_MODULE_UNLOAD",
            switch_event_types_t::SWITCH_EVENT_DTMF => "SWITCH_EVENT_DTMF",
            switch_event_types_t::SWITCH_EVENT_MESSAGE => "SWITCH_EVENT_MESSAGE",
            switch_event_types_t::SWITCH_EVENT_PRESENCE_IN => "SWITCH_EVENT_PRESENCE_IN",
            switch_event_types_t::SWITCH_EVENT_NOTIFY_IN => "SWITCH_EVENT_NOTIFY_IN",
            switch_event_types_t::SWITCH_EVENT_PRESENCE_OUT => "SWITCH_EVENT_PRESENCE_OUT",
            switch_event_types_t::SWITCH_EVENT_PRESENCE_PROBE => "SWITCH_EVENT_PRESENCE_PROBE",
            switch_event_types_t::SWITCH_EVENT_MESSAGE_WAITING => "SWITCH_EVENT_MESSAGE_WAITING",
            switch_event_types_t::SWITCH_EVENT_MESSAGE_QUERY => "SWITCH_EVENT_MESSAGE_QUERY",
            switch_event_types_t::SWITCH_EVENT_ROSTER => "SWITCH_EVENT_ROSTER",
            switch_event_types_t::SWITCH_EVENT_CODEC => "SWITCH_EVENT_CODEC",
            switch_event_types_t::SWITCH_EVENT_BACKGROUND_JOB => "SWITCH_EVENT_BACKGROUND_JOB",
            switch_event_types_t::SWITCH_EVENT_DETECTED_SPEECH => "SWITCH_EVENT_DETECTED_SPEECH",
            switch_event_types_t::SWITCH_EVENT_DETECTED_TONE => "SWITCH_EVENT_DETECTED_TONE",
            switch_event_types_t::SWITCH_EVENT_PRIVATE_COMMAND => "SWITCH_EVENT_PRIVATE_COMMAND",
            switch_event_types_t::SWITCH_EVENT_HEARTBEAT => "SWITCH_EVENT_HEARTBEAT",
            switch_event_types_t::SWITCH_EVENT_TRAP => "SWITCH_EVENT_TRAP",
            switch_event_types_t::SWITCH_EVENT_ADD_SCHEDULE => "SWITCH_EVENT_ADD_SCHEDULE",
            switch_event_types_t::SWITCH_EVENT_DEL_SCHEDULE => "SWITCH_EVENT_DEL_SCHEDULE",
            switch_event_types_t::SWITCH_EVENT_EXE_SCHEDULE => "SWITCH_EVENT_EXE_SCHEDULE",
            switch_event_types_t::SWITCH_EVENT_RE_SCHEDULE => "SWITCH_EVENT_RE_SCHEDULE",
            switch_event_types_t::SWITCH_EVENT_RELOADXML => "SWITCH_EVENT_RELOADXML",
            switch_event_types_t::SWITCH_EVENT_NOTIFY => "SWITCH_EVENT_NOTIFY",
            switch_event_types_t::SWITCH_EVENT_PHONE_FEATURE => "SWITCH_EVENT_PHONE_FEATURE",
            switch_event_types_t::SWITCH_EVENT_PHONE_FEATURE_SUBSCRIBE => {
                "SWITCH_EVENT_PHONE_FEATURE_SUBSCRIBE"
            }
            switch_event_types_t::SWITCH_EVENT_SEND_MESSAGE => "SWITCH_EVENT_SEND_MESSAGE",
            switch_event_types_t::SWITCH_EVENT_RECV_MESSAGE => "SWITCH_EVENT_RECV_MESSAGE",
            switch_event_types_t::SWITCH_EVENT_REQUEST_PARAMS => "SWITCH_EVENT_REQUEST_PARAMS",
            switch_event_types_t::SWITCH_EVENT_CHANNEL_DATA => "SWITCH_EVENT_CHANNEL_DATA",
            switch_event_types_t::SWITCH_EVENT_GENERAL => "SWITCH_EVENT_GENERAL",
            switch_event_types_t::SWITCH_EVENT_COMMAND => "SWITCH_EVENT_COMMAND",
            switch_event_types_t::SWITCH_EVENT_SESSION_HEARTBEAT => {
                "SWITCH_EVENT_SESSION_HEARTBEAT"
            }
            switch_event_types_t::SWITCH_EVENT_CLIENT_DISCONNECTED => {
                "SWITCH_EVENT_CLIENT_DISCONNECTED"
            }
            switch_event_types_t::SWITCH_EVENT_SERVER_DISCONNECTED => {
                "SWITCH_EVENT_SERVER_DISCONNECTED"
            }
            switch_event_types_t::SWITCH_EVENT_SEND_INFO => "SWITCH_EVENT_SEND_INFO",
            switch_event_types_t::SWITCH_EVENT_RECV_INFO => "SWITCH_EVENT_RECV_INFO",
            switch_event_types_t::SWITCH_EVENT_RECV_RTCP_MESSAGE => {
                "SWITCH_EVENT_RECV_RTCP_MESSAGE"
            }
            switch_event_types_t::SWITCH_EVENT_SEND_RTCP_MESSAGE => {
                "SWITCH_EVENT_SEND_RTCP_MESSAGE"
            }
            switch_event_types_t::SWITCH_EVENT_CALL_SECURE => "SWITCH_EVENT_CALL_SECURE",
            switch_event_types_t::SWITCH_EVENT_NAT => "SWITCH_EVENT_NAT",
            switch_event_types_t::SWITCH_EVENT_RECORD_START => "SWITCH_EVENT_RECORD_START",
            switch_event_types_t::SWITCH_EVENT_RECORD_STOP => "SWITCH_EVENT_RECORD_STOP",
            switch_event_types_t::SWITCH_EVENT_PLAYBACK_START => "SWITCH_EVENT_PLAYBACK_START",
            switch_event_types_t::SWITCH_EVENT_PLAYBACK_STOP => "SWITCH_EVENT_PLAYBACK_STOP",
            switch_event_types_t::SWITCH_EVENT_CALL_UPDATE => "SWITCH_EVENT_CALL_UPDATE",
            switch_event_types_t::SWITCH_EVENT_FAILURE => "SWITCH_EVENT_FAILURE",
            switch_event_types_t::SWITCH_EVENT_SOCKET_DATA => "SWITCH_EVENT_SOCKET_DATA",
            switch_event_types_t::SWITCH_EVENT_MEDIA_BUG_START => "SWITCH_EVENT_MEDIA_BUG_START",
            switch_event_types_t::SWITCH_EVENT_MEDIA_BUG_STOP => "SWITCH_EVENT_MEDIA_BUG_STOP",
            switch_event_types_t::SWITCH_EVENT_CONFERENCE_DATA_QUERY => {
                "SWITCH_EVENT_CONFERENCE_DATA_QUERY"
            }
            switch_event_types_t::SWITCH_EVENT_CONFERENCE_DATA => "SWITCH_EVENT_CONFERENCE_DATA",
            switch_event_types_t::SWITCH_EVENT_CALL_SETUP_REQ => "SWITCH_EVENT_CALL_SETUP_REQ",
            switch_event_types_t::SWITCH_EVENT_CALL_SETUP_RESULT => {
                "SWITCH_EVENT_CALL_SETUP_RESULT"
            }
            switch_event_types_t::SWITCH_EVENT_CALL_DETAIL => "SWITCH_EVENT_CALL_DETAIL",
            switch_event_types_t::SWITCH_EVENT_DEVICE_STATE => "SWITCH_EVENT_DEVICE_STATE",
            switch_event_types_t::SWITCH_EVENT_TEXT => "SWITCH_EVENT_TEXT",
            switch_event_types_t::SWITCH_EVENT_SHUTDOWN_REQUESTED => {
                "SWITCH_EVENT_SHUTDOWN_REQUESTED"
            }
            switch_event_types_t::SWITCH_EVENT_ALL => "SWITCH_EVENT_ALL",
            _ => "SWITCH_EVENT_NONE",
        }
    }

    ///_CREATES_AN_ENUM_FROM_FIELD_NAMES_USED_IN_THE_PROTO_BUF_DEFINITION.
    pub fn from_str(value: &str) -> ::core::option::Option<switch_event_types_t> {
        match value {
            "SWITCH_EVENT_CUSTOM" => Some(switch_event_types_t::SWITCH_EVENT_CUSTOM),
            "SWITCH_EVENT_CLONE" => Some(switch_event_types_t::SWITCH_EVENT_CLONE),
            "SWITCH_EVENT_CHANNEL_CREATE" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_CREATE)
            }
            "SWITCH_EVENT_CHANNEL_DESTROY" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_DESTROY)
            }
            "SWITCH_EVENT_CHANNEL_STATE" => Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_STATE),
            "SWITCH_EVENT_CHANNEL_CALLSTATE" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_CALLSTATE)
            }
            "SWITCH_EVENT_CHANNEL_ANSWER" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_ANSWER)
            }
            "SWITCH_EVENT_CHANNEL_HANGUP" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_HANGUP)
            }
            "SWITCH_EVENT_CHANNEL_HANGUP_COMPLETE" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_HANGUP_COMPLETE)
            }
            "SWITCH_EVENT_CHANNEL_EXECUTE" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_EXECUTE)
            }
            "SWITCH_EVENT_CHANNEL_EXECUTE_COMPLETE" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_EXECUTE_COMPLETE)
            }
            "SWITCH_EVENT_CHANNEL_HOLD" => Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_HOLD),
            "SWITCH_EVENT_CHANNEL_UNHOLD" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_UNHOLD)
            }
            "SWITCH_EVENT_CHANNEL_BRIDGE" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_BRIDGE)
            }
            "SWITCH_EVENT_CHANNEL_UNBRIDGE" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_UNBRIDGE)
            }
            "SWITCH_EVENT_CHANNEL_PROGRESS" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_PROGRESS)
            }
            "SWITCH_EVENT_CHANNEL_PROGRESS_MEDIA" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_PROGRESS_MEDIA)
            }
            "SWITCH_EVENT_CHANNEL_OUTGOING" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_OUTGOING)
            }
            "SWITCH_EVENT_CHANNEL_PARK" => Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_PARK),
            "SWITCH_EVENT_CHANNEL_UNPARK" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_UNPARK)
            }
            "SWITCH_EVENT_CHANNEL_APPLICATION" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_APPLICATION)
            }
            "SWITCH_EVENT_CHANNEL_ORIGINATE" => {
                Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_ORIGINATE)
            }
            "SWITCH_EVENT_CHANNEL_UUID" => Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_UUID),
            "SWITCH_EVENT_API" => Some(switch_event_types_t::SWITCH_EVENT_API),
            "SWITCH_EVENT_LOG" => Some(switch_event_types_t::SWITCH_EVENT_LOG),
            "SWITCH_EVENT_INBOUND_CHAN" => Some(switch_event_types_t::SWITCH_EVENT_INBOUND_CHAN),
            "SWITCH_EVENT_OUTBOUND_CHAN" => Some(switch_event_types_t::SWITCH_EVENT_OUTBOUND_CHAN),
            "SWITCH_EVENT_STARTUP" => Some(switch_event_types_t::SWITCH_EVENT_STARTUP),
            "SWITCH_EVENT_SHUTDOWN" => Some(switch_event_types_t::SWITCH_EVENT_SHUTDOWN),
            "SWITCH_EVENT_PUBLISH" => Some(switch_event_types_t::SWITCH_EVENT_PUBLISH),
            "SWITCH_EVENT_UNPUBLISH" => Some(switch_event_types_t::SWITCH_EVENT_UNPUBLISH),
            "SWITCH_EVENT_TALK" => Some(switch_event_types_t::SWITCH_EVENT_TALK),
            "SWITCH_EVENT_NOTALK" => Some(switch_event_types_t::SWITCH_EVENT_NOTALK),
            "SWITCH_EVENT_SESSION_CRASH" => Some(switch_event_types_t::SWITCH_EVENT_SESSION_CRASH),
            "SWITCH_EVENT_MODULE_LOAD" => Some(switch_event_types_t::SWITCH_EVENT_MODULE_LOAD),
            "SWITCH_EVENT_MODULE_UNLOAD" => Some(switch_event_types_t::SWITCH_EVENT_MODULE_UNLOAD),
            "SWITCH_EVENT_DTMF" => Some(switch_event_types_t::SWITCH_EVENT_DTMF),
            "SWITCH_EVENT_MESSAGE" => Some(switch_event_types_t::SWITCH_EVENT_MESSAGE),
            "SWITCH_EVENT_PRESENCE_IN" => Some(switch_event_types_t::SWITCH_EVENT_PRESENCE_IN),
            "SWITCH_EVENT_NOTIFY_IN" => Some(switch_event_types_t::SWITCH_EVENT_NOTIFY_IN),
            "SWITCH_EVENT_PRESENCE_OUT" => Some(switch_event_types_t::SWITCH_EVENT_PRESENCE_OUT),
            "SWITCH_EVENT_PRESENCE_PROBE" => {
                Some(switch_event_types_t::SWITCH_EVENT_PRESENCE_PROBE)
            }
            "SWITCH_EVENT_MESSAGE_WAITING" => {
                Some(switch_event_types_t::SWITCH_EVENT_MESSAGE_WAITING)
            }
            "SWITCH_EVENT_MESSAGE_QUERY" => Some(switch_event_types_t::SWITCH_EVENT_MESSAGE_QUERY),
            "SWITCH_EVENT_ROSTER" => Some(switch_event_types_t::SWITCH_EVENT_ROSTER),
            "SWITCH_EVENT_CODEC" => Some(switch_event_types_t::SWITCH_EVENT_CODEC),
            "SWITCH_EVENT_BACKGROUND_JOB" => {
                Some(switch_event_types_t::SWITCH_EVENT_BACKGROUND_JOB)
            }
            "SWITCH_EVENT_DETECTED_SPEECH" => {
                Some(switch_event_types_t::SWITCH_EVENT_DETECTED_SPEECH)
            }
            "SWITCH_EVENT_DETECTED_TONE" => Some(switch_event_types_t::SWITCH_EVENT_DETECTED_TONE),
            "SWITCH_EVENT_PRIVATE_COMMAND" => {
                Some(switch_event_types_t::SWITCH_EVENT_PRIVATE_COMMAND)
            }
            "SWITCH_EVENT_HEARTBEAT" => Some(switch_event_types_t::SWITCH_EVENT_HEARTBEAT),
            "SWITCH_EVENT_TRAP" => Some(switch_event_types_t::SWITCH_EVENT_TRAP),
            "SWITCH_EVENT_ADD_SCHEDULE" => Some(switch_event_types_t::SWITCH_EVENT_ADD_SCHEDULE),
            "SWITCH_EVENT_DEL_SCHEDULE" => Some(switch_event_types_t::SWITCH_EVENT_DEL_SCHEDULE),
            "SWITCH_EVENT_EXE_SCHEDULE" => Some(switch_event_types_t::SWITCH_EVENT_EXE_SCHEDULE),
            "SWITCH_EVENT_RE_SCHEDULE" => Some(switch_event_types_t::SWITCH_EVENT_RE_SCHEDULE),
            "SWITCH_EVENT_RELOADXML" => Some(switch_event_types_t::SWITCH_EVENT_RELOADXML),
            "SWITCH_EVENT_NOTIFY" => Some(switch_event_types_t::SWITCH_EVENT_NOTIFY),
            "SWITCH_EVENT_PHONE_FEATURE" => Some(switch_event_types_t::SWITCH_EVENT_PHONE_FEATURE),
            "SWITCH_EVENT_PHONE_FEATURE_SUBSCRIBE" => {
                Some(switch_event_types_t::SWITCH_EVENT_PHONE_FEATURE_SUBSCRIBE)
            }
            "SWITCH_EVENT_SEND_MESSAGE" => Some(switch_event_types_t::SWITCH_EVENT_SEND_MESSAGE),
            "SWITCH_EVENT_RECV_MESSAGE" => Some(switch_event_types_t::SWITCH_EVENT_RECV_MESSAGE),
            "SWITCH_EVENT_REQUEST_PARAMS" => {
                Some(switch_event_types_t::SWITCH_EVENT_REQUEST_PARAMS)
            }
            "SWITCH_EVENT_CHANNEL_DATA" => Some(switch_event_types_t::SWITCH_EVENT_CHANNEL_DATA),
            "SWITCH_EVENT_GENERAL" => Some(switch_event_types_t::SWITCH_EVENT_GENERAL),
            "SWITCH_EVENT_COMMAND" => Some(switch_event_types_t::SWITCH_EVENT_COMMAND),
            "SWITCH_EVENT_SESSION_HEARTBEAT" => {
                Some(switch_event_types_t::SWITCH_EVENT_SESSION_HEARTBEAT)
            }
            "SWITCH_EVENT_CLIENT_DISCONNECTED" => {
                Some(switch_event_types_t::SWITCH_EVENT_CLIENT_DISCONNECTED)
            }
            "SWITCH_EVENT_SERVER_DISCONNECTED" => {
                Some(switch_event_types_t::SWITCH_EVENT_SERVER_DISCONNECTED)
            }
            "SWITCH_EVENT_SEND_INFO" => Some(switch_event_types_t::SWITCH_EVENT_SEND_INFO),
            "SWITCH_EVENT_RECV_INFO" => Some(switch_event_types_t::SWITCH_EVENT_RECV_INFO),
            "SWITCH_EVENT_RECV_RTCP_MESSAGE" => {
                Some(switch_event_types_t::SWITCH_EVENT_RECV_RTCP_MESSAGE)
            }
            "SWITCH_EVENT_SEND_RTCP_MESSAGE" => {
                Some(switch_event_types_t::SWITCH_EVENT_SEND_RTCP_MESSAGE)
            }
            "SWITCH_EVENT_CALL_SECURE" => Some(switch_event_types_t::SWITCH_EVENT_CALL_SECURE),
            "SWITCH_EVENT_NAT" => Some(switch_event_types_t::SWITCH_EVENT_NAT),
            "SWITCH_EVENT_RECORD_START" => Some(switch_event_types_t::SWITCH_EVENT_RECORD_START),
            "SWITCH_EVENT_RECORD_STOP" => Some(switch_event_types_t::SWITCH_EVENT_RECORD_STOP),
            "SWITCH_EVENT_PLAYBACK_START" => {
                Some(switch_event_types_t::SWITCH_EVENT_PLAYBACK_START)
            }
            "SWITCH_EVENT_PLAYBACK_STOP" => Some(switch_event_types_t::SWITCH_EVENT_PLAYBACK_STOP),
            "SWITCH_EVENT_CALL_UPDATE" => Some(switch_event_types_t::SWITCH_EVENT_CALL_UPDATE),
            "SWITCH_EVENT_FAILURE" => Some(switch_event_types_t::SWITCH_EVENT_FAILURE),
            "SWITCH_EVENT_SOCKET_DATA" => Some(switch_event_types_t::SWITCH_EVENT_SOCKET_DATA),
            "SWITCH_EVENT_MEDIA_BUG_START" => {
                Some(switch_event_types_t::SWITCH_EVENT_MEDIA_BUG_START)
            }
            "SWITCH_EVENT_MEDIA_BUG_STOP" => {
                Some(switch_event_types_t::SWITCH_EVENT_MEDIA_BUG_STOP)
            }
            "SWITCH_EVENT_CONFERENCE_DATA_QUERY" => {
                Some(switch_event_types_t::SWITCH_EVENT_CONFERENCE_DATA_QUERY)
            }
            "SWITCH_EVENT_CONFERENCE_DATA" => {
                Some(switch_event_types_t::SWITCH_EVENT_CONFERENCE_DATA)
            }
            "SWITCH_EVENT_CALL_SETUP_REQ" => {
                Some(switch_event_types_t::SWITCH_EVENT_CALL_SETUP_REQ)
            }
            "SWITCH_EVENT_CALL_SETUP_RESULT" => {
                Some(switch_event_types_t::SWITCH_EVENT_CALL_SETUP_RESULT)
            }
            "SWITCH_EVENT_CALL_DETAIL" => Some(switch_event_types_t::SWITCH_EVENT_CALL_DETAIL),
            "SWITCH_EVENT_DEVICE_STATE" => Some(switch_event_types_t::SWITCH_EVENT_DEVICE_STATE),
            "SWITCH_EVENT_TEXT" => Some(switch_event_types_t::SWITCH_EVENT_TEXT),
            "SWITCH_EVENT_SHUTDOWN_REQUESTED" => {
                Some(switch_event_types_t::SWITCH_EVENT_SHUTDOWN_REQUESTED)
            }
            "SWITCH_EVENT_ALL" => Some(switch_event_types_t::SWITCH_EVENT_ALL),
            _ => None,
        }
    }
}
