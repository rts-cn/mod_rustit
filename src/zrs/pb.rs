#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Topic {
    #[prost(enumeration = "EventTypes", tag = "1")]
    pub id: i32,
    #[prost(string, tag = "2")]
    pub subclass: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventRequest {
    #[prost(message, repeated, tag = "1")]
    pub topics: ::prost::alloc::vec::Vec<Topic>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(uint32, tag = "1")]
    pub event_id: u32,
    #[prost(uint32, tag = "2")]
    pub priority: u32,
    #[prost(string, tag = "3")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub subclass_name: ::prost::alloc::string::String,
    #[prost(uint64, tag = "5")]
    pub key: u64,
    #[prost(int32, tag = "6")]
    pub flags: i32,
    #[prost(map = "string, string", tag = "7")]
    pub headers: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    #[prost(string, tag = "8")]
    pub body: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventReply {
    #[prost(uint64, tag = "1")]
    pub seq: u64,
    #[prost(message, optional, tag = "2")]
    pub event: ::core::option::Option<Event>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandRequest {
    #[prost(string, tag = "1")]
    pub command: ::prost::alloc::string::String,
    #[prost(bool, tag = "2")]
    pub sync: bool,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandReply {
    #[prost(int32, tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendMsgRequest {
    #[prost(map = "string, string", tag = "1")]
    pub msg: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendMsgReply {
    #[prost(int32, tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum EventTypes {
    SwitchEventCustom = 0,
    SwitchEventClone = 1,
    SwitchEventChannelCreate = 2,
    SwitchEventChannelDestroy = 3,
    SwitchEventChannelState = 4,
    SwitchEventChannelCallstate = 5,
    SwitchEventChannelAnswer = 6,
    SwitchEventChannelHangup = 7,
    SwitchEventChannelHangupComplete = 8,
    SwitchEventChannelExecute = 9,
    SwitchEventChannelExecuteComplete = 10,
    SwitchEventChannelHold = 11,
    SwitchEventChannelUnhold = 12,
    SwitchEventChannelBridge = 13,
    SwitchEventChannelUnbridge = 14,
    SwitchEventChannelProgress = 15,
    SwitchEventChannelProgressMedia = 16,
    SwitchEventChannelOutgoing = 17,
    SwitchEventChannelPark = 18,
    SwitchEventChannelUnpark = 19,
    SwitchEventChannelApplication = 20,
    SwitchEventChannelOriginate = 21,
    SwitchEventChannelUuid = 22,
    SwitchEventApi = 23,
    SwitchEventLog = 24,
    SwitchEventInboundChan = 25,
    SwitchEventOutboundChan = 26,
    SwitchEventStartup = 27,
    SwitchEventShutdown = 28,
    SwitchEventPublish = 29,
    SwitchEventUnpublish = 30,
    SwitchEventTalk = 31,
    SwitchEventNotalk = 32,
    SwitchEventSessionCrash = 33,
    SwitchEventModuleLoad = 34,
    SwitchEventModuleUnload = 35,
    SwitchEventDtmf = 36,
    SwitchEventMessage = 37,
    SwitchEventPresenceIn = 38,
    SwitchEventNotifyIn = 39,
    SwitchEventPresenceOut = 40,
    SwitchEventPresenceProbe = 41,
    SwitchEventMessageWaiting = 42,
    SwitchEventMessageQuery = 43,
    SwitchEventRoster = 44,
    SwitchEventCodec = 45,
    SwitchEventBackgroundJob = 46,
    SwitchEventDetectedSpeech = 47,
    SwitchEventDetectedTone = 48,
    SwitchEventPrivateCommand = 49,
    SwitchEventHeartbeat = 50,
    SwitchEventTrap = 51,
    SwitchEventAddSchedule = 52,
    SwitchEventDelSchedule = 53,
    SwitchEventExeSchedule = 54,
    SwitchEventReSchedule = 55,
    SwitchEventReloadxml = 56,
    SwitchEventNotify = 57,
    SwitchEventPhoneFeature = 58,
    SwitchEventPhoneFeatureSubscribe = 59,
    SwitchEventSendMessage = 60,
    SwitchEventRecvMessage = 61,
    SwitchEventRequestParams = 62,
    SwitchEventChannelData = 63,
    SwitchEventGeneral = 64,
    SwitchEventCommand = 65,
    SwitchEventSessionHeartbeat = 66,
    SwitchEventClientDisconnected = 67,
    SwitchEventServerDisconnected = 68,
    SwitchEventSendInfo = 69,
    SwitchEventRecvInfo = 70,
    SwitchEventRecvRtcpMessage = 71,
    SwitchEventSendRtcpMessage = 72,
    SwitchEventCallSecure = 73,
    SwitchEventNat = 74,
    SwitchEventRecordStart = 75,
    SwitchEventRecordStop = 76,
    SwitchEventPlaybackStart = 77,
    SwitchEventPlaybackStop = 78,
    SwitchEventCallUpdate = 79,
    SwitchEventFailure = 80,
    SwitchEventSocketData = 81,
    SwitchEventMediaBugStart = 82,
    SwitchEventMediaBugStop = 83,
    SwitchEventConferenceDataQuery = 84,
    SwitchEventConferenceData = 85,
    SwitchEventCallSetupReq = 86,
    SwitchEventCallSetupResult = 87,
    SwitchEventCallDetail = 88,
    SwitchEventDeviceState = 89,
    SwitchEventText = 90,
    SwitchEventShutdownRequested = 91,
    SwitchEventAll = 92,
}
impl EventTypes {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            EventTypes::SwitchEventCustom => "SWITCH_EVENT_CUSTOM",
            EventTypes::SwitchEventClone => "SWITCH_EVENT_CLONE",
            EventTypes::SwitchEventChannelCreate => "SWITCH_EVENT_CHANNEL_CREATE",
            EventTypes::SwitchEventChannelDestroy => "SWITCH_EVENT_CHANNEL_DESTROY",
            EventTypes::SwitchEventChannelState => "SWITCH_EVENT_CHANNEL_STATE",
            EventTypes::SwitchEventChannelCallstate => "SWITCH_EVENT_CHANNEL_CALLSTATE",
            EventTypes::SwitchEventChannelAnswer => "SWITCH_EVENT_CHANNEL_ANSWER",
            EventTypes::SwitchEventChannelHangup => "SWITCH_EVENT_CHANNEL_HANGUP",
            EventTypes::SwitchEventChannelHangupComplete => {
                "SWITCH_EVENT_CHANNEL_HANGUP_COMPLETE"
            }
            EventTypes::SwitchEventChannelExecute => "SWITCH_EVENT_CHANNEL_EXECUTE",
            EventTypes::SwitchEventChannelExecuteComplete => {
                "SWITCH_EVENT_CHANNEL_EXECUTE_COMPLETE"
            }
            EventTypes::SwitchEventChannelHold => "SWITCH_EVENT_CHANNEL_HOLD",
            EventTypes::SwitchEventChannelUnhold => "SWITCH_EVENT_CHANNEL_UNHOLD",
            EventTypes::SwitchEventChannelBridge => "SWITCH_EVENT_CHANNEL_BRIDGE",
            EventTypes::SwitchEventChannelUnbridge => "SWITCH_EVENT_CHANNEL_UNBRIDGE",
            EventTypes::SwitchEventChannelProgress => "SWITCH_EVENT_CHANNEL_PROGRESS",
            EventTypes::SwitchEventChannelProgressMedia => {
                "SWITCH_EVENT_CHANNEL_PROGRESS_MEDIA"
            }
            EventTypes::SwitchEventChannelOutgoing => "SWITCH_EVENT_CHANNEL_OUTGOING",
            EventTypes::SwitchEventChannelPark => "SWITCH_EVENT_CHANNEL_PARK",
            EventTypes::SwitchEventChannelUnpark => "SWITCH_EVENT_CHANNEL_UNPARK",
            EventTypes::SwitchEventChannelApplication => {
                "SWITCH_EVENT_CHANNEL_APPLICATION"
            }
            EventTypes::SwitchEventChannelOriginate => "SWITCH_EVENT_CHANNEL_ORIGINATE",
            EventTypes::SwitchEventChannelUuid => "SWITCH_EVENT_CHANNEL_UUID",
            EventTypes::SwitchEventApi => "SWITCH_EVENT_API",
            EventTypes::SwitchEventLog => "SWITCH_EVENT_LOG",
            EventTypes::SwitchEventInboundChan => "SWITCH_EVENT_INBOUND_CHAN",
            EventTypes::SwitchEventOutboundChan => "SWITCH_EVENT_OUTBOUND_CHAN",
            EventTypes::SwitchEventStartup => "SWITCH_EVENT_STARTUP",
            EventTypes::SwitchEventShutdown => "SWITCH_EVENT_SHUTDOWN",
            EventTypes::SwitchEventPublish => "SWITCH_EVENT_PUBLISH",
            EventTypes::SwitchEventUnpublish => "SWITCH_EVENT_UNPUBLISH",
            EventTypes::SwitchEventTalk => "SWITCH_EVENT_TALK",
            EventTypes::SwitchEventNotalk => "SWITCH_EVENT_NOTALK",
            EventTypes::SwitchEventSessionCrash => "SWITCH_EVENT_SESSION_CRASH",
            EventTypes::SwitchEventModuleLoad => "SWITCH_EVENT_MODULE_LOAD",
            EventTypes::SwitchEventModuleUnload => "SWITCH_EVENT_MODULE_UNLOAD",
            EventTypes::SwitchEventDtmf => "SWITCH_EVENT_DTMF",
            EventTypes::SwitchEventMessage => "SWITCH_EVENT_MESSAGE",
            EventTypes::SwitchEventPresenceIn => "SWITCH_EVENT_PRESENCE_IN",
            EventTypes::SwitchEventNotifyIn => "SWITCH_EVENT_NOTIFY_IN",
            EventTypes::SwitchEventPresenceOut => "SWITCH_EVENT_PRESENCE_OUT",
            EventTypes::SwitchEventPresenceProbe => "SWITCH_EVENT_PRESENCE_PROBE",
            EventTypes::SwitchEventMessageWaiting => "SWITCH_EVENT_MESSAGE_WAITING",
            EventTypes::SwitchEventMessageQuery => "SWITCH_EVENT_MESSAGE_QUERY",
            EventTypes::SwitchEventRoster => "SWITCH_EVENT_ROSTER",
            EventTypes::SwitchEventCodec => "SWITCH_EVENT_CODEC",
            EventTypes::SwitchEventBackgroundJob => "SWITCH_EVENT_BACKGROUND_JOB",
            EventTypes::SwitchEventDetectedSpeech => "SWITCH_EVENT_DETECTED_SPEECH",
            EventTypes::SwitchEventDetectedTone => "SWITCH_EVENT_DETECTED_TONE",
            EventTypes::SwitchEventPrivateCommand => "SWITCH_EVENT_PRIVATE_COMMAND",
            EventTypes::SwitchEventHeartbeat => "SWITCH_EVENT_HEARTBEAT",
            EventTypes::SwitchEventTrap => "SWITCH_EVENT_TRAP",
            EventTypes::SwitchEventAddSchedule => "SWITCH_EVENT_ADD_SCHEDULE",
            EventTypes::SwitchEventDelSchedule => "SWITCH_EVENT_DEL_SCHEDULE",
            EventTypes::SwitchEventExeSchedule => "SWITCH_EVENT_EXE_SCHEDULE",
            EventTypes::SwitchEventReSchedule => "SWITCH_EVENT_RE_SCHEDULE",
            EventTypes::SwitchEventReloadxml => "SWITCH_EVENT_RELOADXML",
            EventTypes::SwitchEventNotify => "SWITCH_EVENT_NOTIFY",
            EventTypes::SwitchEventPhoneFeature => "SWITCH_EVENT_PHONE_FEATURE",
            EventTypes::SwitchEventPhoneFeatureSubscribe => {
                "SWITCH_EVENT_PHONE_FEATURE_SUBSCRIBE"
            }
            EventTypes::SwitchEventSendMessage => "SWITCH_EVENT_SEND_MESSAGE",
            EventTypes::SwitchEventRecvMessage => "SWITCH_EVENT_RECV_MESSAGE",
            EventTypes::SwitchEventRequestParams => "SWITCH_EVENT_REQUEST_PARAMS",
            EventTypes::SwitchEventChannelData => "SWITCH_EVENT_CHANNEL_DATA",
            EventTypes::SwitchEventGeneral => "SWITCH_EVENT_GENERAL",
            EventTypes::SwitchEventCommand => "SWITCH_EVENT_COMMAND",
            EventTypes::SwitchEventSessionHeartbeat => "SWITCH_EVENT_SESSION_HEARTBEAT",
            EventTypes::SwitchEventClientDisconnected => {
                "SWITCH_EVENT_CLIENT_DISCONNECTED"
            }
            EventTypes::SwitchEventServerDisconnected => {
                "SWITCH_EVENT_SERVER_DISCONNECTED"
            }
            EventTypes::SwitchEventSendInfo => "SWITCH_EVENT_SEND_INFO",
            EventTypes::SwitchEventRecvInfo => "SWITCH_EVENT_RECV_INFO",
            EventTypes::SwitchEventRecvRtcpMessage => "SWITCH_EVENT_RECV_RTCP_MESSAGE",
            EventTypes::SwitchEventSendRtcpMessage => "SWITCH_EVENT_SEND_RTCP_MESSAGE",
            EventTypes::SwitchEventCallSecure => "SWITCH_EVENT_CALL_SECURE",
            EventTypes::SwitchEventNat => "SWITCH_EVENT_NAT",
            EventTypes::SwitchEventRecordStart => "SWITCH_EVENT_RECORD_START",
            EventTypes::SwitchEventRecordStop => "SWITCH_EVENT_RECORD_STOP",
            EventTypes::SwitchEventPlaybackStart => "SWITCH_EVENT_PLAYBACK_START",
            EventTypes::SwitchEventPlaybackStop => "SWITCH_EVENT_PLAYBACK_STOP",
            EventTypes::SwitchEventCallUpdate => "SWITCH_EVENT_CALL_UPDATE",
            EventTypes::SwitchEventFailure => "SWITCH_EVENT_FAILURE",
            EventTypes::SwitchEventSocketData => "SWITCH_EVENT_SOCKET_DATA",
            EventTypes::SwitchEventMediaBugStart => "SWITCH_EVENT_MEDIA_BUG_START",
            EventTypes::SwitchEventMediaBugStop => "SWITCH_EVENT_MEDIA_BUG_STOP",
            EventTypes::SwitchEventConferenceDataQuery => {
                "SWITCH_EVENT_CONFERENCE_DATA_QUERY"
            }
            EventTypes::SwitchEventConferenceData => "SWITCH_EVENT_CONFERENCE_DATA",
            EventTypes::SwitchEventCallSetupReq => "SWITCH_EVENT_CALL_SETUP_REQ",
            EventTypes::SwitchEventCallSetupResult => "SWITCH_EVENT_CALL_SETUP_RESULT",
            EventTypes::SwitchEventCallDetail => "SWITCH_EVENT_CALL_DETAIL",
            EventTypes::SwitchEventDeviceState => "SWITCH_EVENT_DEVICE_STATE",
            EventTypes::SwitchEventText => "SWITCH_EVENT_TEXT",
            EventTypes::SwitchEventShutdownRequested => "SWITCH_EVENT_SHUTDOWN_REQUESTED",
            EventTypes::SwitchEventAll => "SWITCH_EVENT_ALL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SWITCH_EVENT_CUSTOM" => Some(Self::SwitchEventCustom),
            "SWITCH_EVENT_CLONE" => Some(Self::SwitchEventClone),
            "SWITCH_EVENT_CHANNEL_CREATE" => Some(Self::SwitchEventChannelCreate),
            "SWITCH_EVENT_CHANNEL_DESTROY" => Some(Self::SwitchEventChannelDestroy),
            "SWITCH_EVENT_CHANNEL_STATE" => Some(Self::SwitchEventChannelState),
            "SWITCH_EVENT_CHANNEL_CALLSTATE" => Some(Self::SwitchEventChannelCallstate),
            "SWITCH_EVENT_CHANNEL_ANSWER" => Some(Self::SwitchEventChannelAnswer),
            "SWITCH_EVENT_CHANNEL_HANGUP" => Some(Self::SwitchEventChannelHangup),
            "SWITCH_EVENT_CHANNEL_HANGUP_COMPLETE" => {
                Some(Self::SwitchEventChannelHangupComplete)
            }
            "SWITCH_EVENT_CHANNEL_EXECUTE" => Some(Self::SwitchEventChannelExecute),
            "SWITCH_EVENT_CHANNEL_EXECUTE_COMPLETE" => {
                Some(Self::SwitchEventChannelExecuteComplete)
            }
            "SWITCH_EVENT_CHANNEL_HOLD" => Some(Self::SwitchEventChannelHold),
            "SWITCH_EVENT_CHANNEL_UNHOLD" => Some(Self::SwitchEventChannelUnhold),
            "SWITCH_EVENT_CHANNEL_BRIDGE" => Some(Self::SwitchEventChannelBridge),
            "SWITCH_EVENT_CHANNEL_UNBRIDGE" => Some(Self::SwitchEventChannelUnbridge),
            "SWITCH_EVENT_CHANNEL_PROGRESS" => Some(Self::SwitchEventChannelProgress),
            "SWITCH_EVENT_CHANNEL_PROGRESS_MEDIA" => {
                Some(Self::SwitchEventChannelProgressMedia)
            }
            "SWITCH_EVENT_CHANNEL_OUTGOING" => Some(Self::SwitchEventChannelOutgoing),
            "SWITCH_EVENT_CHANNEL_PARK" => Some(Self::SwitchEventChannelPark),
            "SWITCH_EVENT_CHANNEL_UNPARK" => Some(Self::SwitchEventChannelUnpark),
            "SWITCH_EVENT_CHANNEL_APPLICATION" => {
                Some(Self::SwitchEventChannelApplication)
            }
            "SWITCH_EVENT_CHANNEL_ORIGINATE" => Some(Self::SwitchEventChannelOriginate),
            "SWITCH_EVENT_CHANNEL_UUID" => Some(Self::SwitchEventChannelUuid),
            "SWITCH_EVENT_API" => Some(Self::SwitchEventApi),
            "SWITCH_EVENT_LOG" => Some(Self::SwitchEventLog),
            "SWITCH_EVENT_INBOUND_CHAN" => Some(Self::SwitchEventInboundChan),
            "SWITCH_EVENT_OUTBOUND_CHAN" => Some(Self::SwitchEventOutboundChan),
            "SWITCH_EVENT_STARTUP" => Some(Self::SwitchEventStartup),
            "SWITCH_EVENT_SHUTDOWN" => Some(Self::SwitchEventShutdown),
            "SWITCH_EVENT_PUBLISH" => Some(Self::SwitchEventPublish),
            "SWITCH_EVENT_UNPUBLISH" => Some(Self::SwitchEventUnpublish),
            "SWITCH_EVENT_TALK" => Some(Self::SwitchEventTalk),
            "SWITCH_EVENT_NOTALK" => Some(Self::SwitchEventNotalk),
            "SWITCH_EVENT_SESSION_CRASH" => Some(Self::SwitchEventSessionCrash),
            "SWITCH_EVENT_MODULE_LOAD" => Some(Self::SwitchEventModuleLoad),
            "SWITCH_EVENT_MODULE_UNLOAD" => Some(Self::SwitchEventModuleUnload),
            "SWITCH_EVENT_DTMF" => Some(Self::SwitchEventDtmf),
            "SWITCH_EVENT_MESSAGE" => Some(Self::SwitchEventMessage),
            "SWITCH_EVENT_PRESENCE_IN" => Some(Self::SwitchEventPresenceIn),
            "SWITCH_EVENT_NOTIFY_IN" => Some(Self::SwitchEventNotifyIn),
            "SWITCH_EVENT_PRESENCE_OUT" => Some(Self::SwitchEventPresenceOut),
            "SWITCH_EVENT_PRESENCE_PROBE" => Some(Self::SwitchEventPresenceProbe),
            "SWITCH_EVENT_MESSAGE_WAITING" => Some(Self::SwitchEventMessageWaiting),
            "SWITCH_EVENT_MESSAGE_QUERY" => Some(Self::SwitchEventMessageQuery),
            "SWITCH_EVENT_ROSTER" => Some(Self::SwitchEventRoster),
            "SWITCH_EVENT_CODEC" => Some(Self::SwitchEventCodec),
            "SWITCH_EVENT_BACKGROUND_JOB" => Some(Self::SwitchEventBackgroundJob),
            "SWITCH_EVENT_DETECTED_SPEECH" => Some(Self::SwitchEventDetectedSpeech),
            "SWITCH_EVENT_DETECTED_TONE" => Some(Self::SwitchEventDetectedTone),
            "SWITCH_EVENT_PRIVATE_COMMAND" => Some(Self::SwitchEventPrivateCommand),
            "SWITCH_EVENT_HEARTBEAT" => Some(Self::SwitchEventHeartbeat),
            "SWITCH_EVENT_TRAP" => Some(Self::SwitchEventTrap),
            "SWITCH_EVENT_ADD_SCHEDULE" => Some(Self::SwitchEventAddSchedule),
            "SWITCH_EVENT_DEL_SCHEDULE" => Some(Self::SwitchEventDelSchedule),
            "SWITCH_EVENT_EXE_SCHEDULE" => Some(Self::SwitchEventExeSchedule),
            "SWITCH_EVENT_RE_SCHEDULE" => Some(Self::SwitchEventReSchedule),
            "SWITCH_EVENT_RELOADXML" => Some(Self::SwitchEventReloadxml),
            "SWITCH_EVENT_NOTIFY" => Some(Self::SwitchEventNotify),
            "SWITCH_EVENT_PHONE_FEATURE" => Some(Self::SwitchEventPhoneFeature),
            "SWITCH_EVENT_PHONE_FEATURE_SUBSCRIBE" => {
                Some(Self::SwitchEventPhoneFeatureSubscribe)
            }
            "SWITCH_EVENT_SEND_MESSAGE" => Some(Self::SwitchEventSendMessage),
            "SWITCH_EVENT_RECV_MESSAGE" => Some(Self::SwitchEventRecvMessage),
            "SWITCH_EVENT_REQUEST_PARAMS" => Some(Self::SwitchEventRequestParams),
            "SWITCH_EVENT_CHANNEL_DATA" => Some(Self::SwitchEventChannelData),
            "SWITCH_EVENT_GENERAL" => Some(Self::SwitchEventGeneral),
            "SWITCH_EVENT_COMMAND" => Some(Self::SwitchEventCommand),
            "SWITCH_EVENT_SESSION_HEARTBEAT" => Some(Self::SwitchEventSessionHeartbeat),
            "SWITCH_EVENT_CLIENT_DISCONNECTED" => {
                Some(Self::SwitchEventClientDisconnected)
            }
            "SWITCH_EVENT_SERVER_DISCONNECTED" => {
                Some(Self::SwitchEventServerDisconnected)
            }
            "SWITCH_EVENT_SEND_INFO" => Some(Self::SwitchEventSendInfo),
            "SWITCH_EVENT_RECV_INFO" => Some(Self::SwitchEventRecvInfo),
            "SWITCH_EVENT_RECV_RTCP_MESSAGE" => Some(Self::SwitchEventRecvRtcpMessage),
            "SWITCH_EVENT_SEND_RTCP_MESSAGE" => Some(Self::SwitchEventSendRtcpMessage),
            "SWITCH_EVENT_CALL_SECURE" => Some(Self::SwitchEventCallSecure),
            "SWITCH_EVENT_NAT" => Some(Self::SwitchEventNat),
            "SWITCH_EVENT_RECORD_START" => Some(Self::SwitchEventRecordStart),
            "SWITCH_EVENT_RECORD_STOP" => Some(Self::SwitchEventRecordStop),
            "SWITCH_EVENT_PLAYBACK_START" => Some(Self::SwitchEventPlaybackStart),
            "SWITCH_EVENT_PLAYBACK_STOP" => Some(Self::SwitchEventPlaybackStop),
            "SWITCH_EVENT_CALL_UPDATE" => Some(Self::SwitchEventCallUpdate),
            "SWITCH_EVENT_FAILURE" => Some(Self::SwitchEventFailure),
            "SWITCH_EVENT_SOCKET_DATA" => Some(Self::SwitchEventSocketData),
            "SWITCH_EVENT_MEDIA_BUG_START" => Some(Self::SwitchEventMediaBugStart),
            "SWITCH_EVENT_MEDIA_BUG_STOP" => Some(Self::SwitchEventMediaBugStop),
            "SWITCH_EVENT_CONFERENCE_DATA_QUERY" => {
                Some(Self::SwitchEventConferenceDataQuery)
            }
            "SWITCH_EVENT_CONFERENCE_DATA" => Some(Self::SwitchEventConferenceData),
            "SWITCH_EVENT_CALL_SETUP_REQ" => Some(Self::SwitchEventCallSetupReq),
            "SWITCH_EVENT_CALL_SETUP_RESULT" => Some(Self::SwitchEventCallSetupResult),
            "SWITCH_EVENT_CALL_DETAIL" => Some(Self::SwitchEventCallDetail),
            "SWITCH_EVENT_DEVICE_STATE" => Some(Self::SwitchEventDeviceState),
            "SWITCH_EVENT_TEXT" => Some(Self::SwitchEventText),
            "SWITCH_EVENT_SHUTDOWN_REQUESTED" => Some(Self::SwitchEventShutdownRequested),
            "SWITCH_EVENT_ALL" => Some(Self::SwitchEventAll),
            _ => None,
        }
    }
}
/// Generated server implementations.
pub mod zr_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ZrServer.
    #[async_trait]
    pub trait Zr: Send + Sync + 'static {
        /// Server streaming response type for the Event method.
        type EventStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::EventReply, tonic::Status>,
            >
            + Send
            + 'static;
        /// Subscribe the FreeSWITCH events
        async fn event(
            &self,
            request: tonic::Request<super::EventRequest>,
        ) -> std::result::Result<tonic::Response<Self::EventStream>, tonic::Status>;
        /// Command sends a single command to the server and returns a response Event.
        async fn command(
            &self,
            request: tonic::Request<super::CommandRequest>,
        ) -> std::result::Result<tonic::Response<super::CommandReply>, tonic::Status>;
        /// SendMsg sends messages to FreeSWITCH and returns a response Event.
        async fn send_msg(
            &self,
            request: tonic::Request<super::SendMsgRequest>,
        ) -> std::result::Result<tonic::Response<super::SendMsgReply>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ZrServer<T: Zr> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Zr> ZrServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ZrServer<T>
    where
        T: Zr,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/pb.zr/Event" => {
                    #[allow(non_camel_case_types)]
                    struct EventSvc<T: Zr>(pub Arc<T>);
                    impl<
                        T: Zr,
                    > tonic::server::ServerStreamingService<super::EventRequest>
                    for EventSvc<T> {
                        type Response = super::EventReply;
                        type ResponseStream = T::EventStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EventRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Zr>::event(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EventSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/pb.zr/Command" => {
                    #[allow(non_camel_case_types)]
                    struct CommandSvc<T: Zr>(pub Arc<T>);
                    impl<T: Zr> tonic::server::UnaryService<super::CommandRequest>
                    for CommandSvc<T> {
                        type Response = super::CommandReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CommandRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Zr>::command(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CommandSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/pb.zr/SendMsg" => {
                    #[allow(non_camel_case_types)]
                    struct SendMsgSvc<T: Zr>(pub Arc<T>);
                    impl<T: Zr> tonic::server::UnaryService<super::SendMsgRequest>
                    for SendMsgSvc<T> {
                        type Response = super::SendMsgReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SendMsgRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Zr>::send_msg(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendMsgSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Zr> Clone for ZrServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Zr> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Zr> tonic::server::NamedService for ZrServer<T> {
        const NAME: &'static str = "pb.zr";
    }
}
