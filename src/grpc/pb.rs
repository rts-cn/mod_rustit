#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReloadXmlRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReloadAclRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModRequest {
    #[prost(string, tag = "1")]
    pub mod_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Reply {
    #[prost(int32, tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnRegisterRequest {
    #[prost(string, tag = "1")]
    pub uuid: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Topic {
    #[prost(string, tag = "1")]
    pub event_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub subclass: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequest {
    #[prost(message, repeated, tag = "1")]
    pub topics: ::prost::alloc::vec::Vec<Topic>,
}
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandRequest {
    #[prost(string, tag = "1")]
    pub command: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub args: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendMsgRequest {
    #[prost(string, tag = "1")]
    pub uuid: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "4")]
    pub headers: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendEventRequest {
    #[prost(uint32, tag = "1")]
    pub event_id: u32,
    #[prost(string, tag = "2")]
    pub subclass_name: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "3")]
    pub headers: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    #[prost(string, tag = "4")]
    pub body: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SystemStatus {
    #[prost(int64, tag = "1")]
    pub uptime: i64,
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
    #[prost(bool, tag = "3")]
    pub ready: bool,
    #[prost(uint64, tag = "4")]
    pub session_total: u64,
    #[prost(uint32, tag = "5")]
    pub session_active: u32,
    #[prost(int32, tag = "6")]
    pub session_peak: i32,
    #[prost(int32, tag = "7")]
    pub session_peak_5min: i32,
    #[prost(uint32, tag = "8")]
    pub session_limit: u32,
    #[prost(int32, tag = "9")]
    pub rate_current: i32,
    #[prost(int32, tag = "10")]
    pub rate_max: i32,
    #[prost(int32, tag = "11")]
    pub rate_peak: i32,
    #[prost(int32, tag = "12")]
    pub rate_peak_5min: i32,
    #[prost(double, tag = "13")]
    pub idle_cpu_used: f64,
    #[prost(double, tag = "14")]
    pub idle_cpu_allowed: f64,
    #[prost(float, tag = "15")]
    pub stack_size_current: f32,
    #[prost(float, tag = "16")]
    pub stack_size_max: f32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusReply {
    #[prost(int32, tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub status: ::core::option::Option<SystemStatus>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JsapiRequest {
    #[prost(string, tag = "1")]
    pub command: ::prost::alloc::string::String,
}
/// Generated client implementations.
pub mod switch_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct SwitchClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl SwitchClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> SwitchClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> SwitchClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            SwitchClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Subscribe the events
        pub async fn subscribe(
            &mut self,
            request: impl tonic::IntoRequest<super::SubscribeRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::Event>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pb.switch/Subscribe");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("pb.switch", "Subscribe"));
            self.inner.server_streaming(req, path, codec).await
        }
        /// Command sends a single command to the server and returns a response Event.
        pub async fn command(
            &mut self,
            request: impl tonic::IntoRequest<super::CommandRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pb.switch/Command");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("pb.switch", "Command"));
            self.inner.unary(req, path, codec).await
        }
        /// SendMsg sends messages and returns a response Event.
        pub async fn send_msg(
            &mut self,
            request: impl tonic::IntoRequest<super::SendMsgRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pb.switch/SendMsg");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("pb.switch", "SendMsg"));
            self.inner.unary(req, path, codec).await
        }
        /// SendEvent sends event
        pub async fn send_event(
            &mut self,
            request: impl tonic::IntoRequest<super::SendEventRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pb.switch/SendEvent");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("pb.switch", "SendEvent"));
            self.inner.unary(req, path, codec).await
        }
        /// Reload xml
        pub async fn reload_xml(
            &mut self,
            request: impl tonic::IntoRequest<super::ReloadXmlRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pb.switch/ReloadXML");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("pb.switch", "ReloadXML"));
            self.inner.unary(req, path, codec).await
        }
        /// Reload Acl
        pub async fn reload_acl(
            &mut self,
            request: impl tonic::IntoRequest<super::ReloadAclRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pb.switch/ReloadAcl");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("pb.switch", "ReloadAcl"));
            self.inner.unary(req, path, codec).await
        }
        /// Reload mod
        pub async fn reload_mod(
            &mut self,
            request: impl tonic::IntoRequest<super::ModRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pb.switch/ReloadMod");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("pb.switch", "ReloadMod"));
            self.inner.unary(req, path, codec).await
        }
        /// Load mod
        pub async fn load_mod(
            &mut self,
            request: impl tonic::IntoRequest<super::ModRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pb.switch/LoadMod");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("pb.switch", "LoadMod"));
            self.inner.unary(req, path, codec).await
        }
        /// Unload mod
        pub async fn unload_mod(
            &mut self,
            request: impl tonic::IntoRequest<super::ModRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pb.switch/UnloadMod");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("pb.switch", "UnloadMod"));
            self.inner.unary(req, path, codec).await
        }
        /// JSAPI
        pub async fn jsapi(
            &mut self,
            request: impl tonic::IntoRequest<super::JsapiRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/pb.switch/JSAPI");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("pb.switch", "JSAPI"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod switch_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with SwitchServer.
    #[async_trait]
    pub trait Switch: Send + Sync + 'static {
        /// Server streaming response type for the Subscribe method.
        type SubscribeStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::Event, tonic::Status>,
            >
            + Send
            + 'static;
        /// Subscribe the events
        async fn subscribe(
            &self,
            request: tonic::Request<super::SubscribeRequest>,
        ) -> std::result::Result<tonic::Response<Self::SubscribeStream>, tonic::Status>;
        /// Command sends a single command to the server and returns a response Event.
        async fn command(
            &self,
            request: tonic::Request<super::CommandRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status>;
        /// SendMsg sends messages and returns a response Event.
        async fn send_msg(
            &self,
            request: tonic::Request<super::SendMsgRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status>;
        /// SendEvent sends event
        async fn send_event(
            &self,
            request: tonic::Request<super::SendEventRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status>;
        /// Reload xml
        async fn reload_xml(
            &self,
            request: tonic::Request<super::ReloadXmlRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status>;
        /// Reload Acl
        async fn reload_acl(
            &self,
            request: tonic::Request<super::ReloadAclRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status>;
        /// Reload mod
        async fn reload_mod(
            &self,
            request: tonic::Request<super::ModRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status>;
        /// Load mod
        async fn load_mod(
            &self,
            request: tonic::Request<super::ModRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status>;
        /// Unload mod
        async fn unload_mod(
            &self,
            request: tonic::Request<super::ModRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status>;
        /// JSAPI
        async fn jsapi(
            &self,
            request: tonic::Request<super::JsapiRequest>,
        ) -> std::result::Result<tonic::Response<super::Reply>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct SwitchServer<T: Switch> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Switch> SwitchServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for SwitchServer<T>
    where
        T: Switch,
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
                "/pb.switch/Subscribe" => {
                    #[allow(non_camel_case_types)]
                    struct SubscribeSvc<T: Switch>(pub Arc<T>);
                    impl<
                        T: Switch,
                    > tonic::server::ServerStreamingService<super::SubscribeRequest>
                    for SubscribeSvc<T> {
                        type Response = super::Event;
                        type ResponseStream = T::SubscribeStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SubscribeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Switch>::subscribe(&inner, request).await
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
                        let method = SubscribeSvc(inner);
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
                "/pb.switch/Command" => {
                    #[allow(non_camel_case_types)]
                    struct CommandSvc<T: Switch>(pub Arc<T>);
                    impl<T: Switch> tonic::server::UnaryService<super::CommandRequest>
                    for CommandSvc<T> {
                        type Response = super::Reply;
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
                                <T as Switch>::command(&inner, request).await
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
                "/pb.switch/SendMsg" => {
                    #[allow(non_camel_case_types)]
                    struct SendMsgSvc<T: Switch>(pub Arc<T>);
                    impl<T: Switch> tonic::server::UnaryService<super::SendMsgRequest>
                    for SendMsgSvc<T> {
                        type Response = super::Reply;
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
                                <T as Switch>::send_msg(&inner, request).await
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
                "/pb.switch/SendEvent" => {
                    #[allow(non_camel_case_types)]
                    struct SendEventSvc<T: Switch>(pub Arc<T>);
                    impl<T: Switch> tonic::server::UnaryService<super::SendEventRequest>
                    for SendEventSvc<T> {
                        type Response = super::Reply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SendEventRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Switch>::send_event(&inner, request).await
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
                        let method = SendEventSvc(inner);
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
                "/pb.switch/ReloadXML" => {
                    #[allow(non_camel_case_types)]
                    struct ReloadXMLSvc<T: Switch>(pub Arc<T>);
                    impl<T: Switch> tonic::server::UnaryService<super::ReloadXmlRequest>
                    for ReloadXMLSvc<T> {
                        type Response = super::Reply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReloadXmlRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Switch>::reload_xml(&inner, request).await
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
                        let method = ReloadXMLSvc(inner);
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
                "/pb.switch/ReloadAcl" => {
                    #[allow(non_camel_case_types)]
                    struct ReloadAclSvc<T: Switch>(pub Arc<T>);
                    impl<T: Switch> tonic::server::UnaryService<super::ReloadAclRequest>
                    for ReloadAclSvc<T> {
                        type Response = super::Reply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReloadAclRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Switch>::reload_acl(&inner, request).await
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
                        let method = ReloadAclSvc(inner);
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
                "/pb.switch/ReloadMod" => {
                    #[allow(non_camel_case_types)]
                    struct ReloadModSvc<T: Switch>(pub Arc<T>);
                    impl<T: Switch> tonic::server::UnaryService<super::ModRequest>
                    for ReloadModSvc<T> {
                        type Response = super::Reply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ModRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Switch>::reload_mod(&inner, request).await
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
                        let method = ReloadModSvc(inner);
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
                "/pb.switch/LoadMod" => {
                    #[allow(non_camel_case_types)]
                    struct LoadModSvc<T: Switch>(pub Arc<T>);
                    impl<T: Switch> tonic::server::UnaryService<super::ModRequest>
                    for LoadModSvc<T> {
                        type Response = super::Reply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ModRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Switch>::load_mod(&inner, request).await
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
                        let method = LoadModSvc(inner);
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
                "/pb.switch/UnloadMod" => {
                    #[allow(non_camel_case_types)]
                    struct UnloadModSvc<T: Switch>(pub Arc<T>);
                    impl<T: Switch> tonic::server::UnaryService<super::ModRequest>
                    for UnloadModSvc<T> {
                        type Response = super::Reply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ModRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Switch>::unload_mod(&inner, request).await
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
                        let method = UnloadModSvc(inner);
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
                "/pb.switch/JSAPI" => {
                    #[allow(non_camel_case_types)]
                    struct JSAPISvc<T: Switch>(pub Arc<T>);
                    impl<T: Switch> tonic::server::UnaryService<super::JsapiRequest>
                    for JSAPISvc<T> {
                        type Response = super::Reply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::JsapiRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Switch>::jsapi(&inner, request).await
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
                        let method = JSAPISvc(inner);
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
    impl<T: Switch> Clone for SwitchServer<T> {
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
    impl<T: Switch> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Switch> tonic::server::NamedService for SwitchServer<T> {
        const NAME: &'static str = "pb.switch";
    }
}
