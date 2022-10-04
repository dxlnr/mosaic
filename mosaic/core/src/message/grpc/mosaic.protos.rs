/// 
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DataType {
    /// Not a legal value for DataType.  Used to indicate a DataType field
    /// has not been set.
    DtInvalid = 0,
    /// Data types that all computation devices are expected to be
    /// capable to support.
    DtF16 = 1,
    DtF32 = 2,
    DtF64 = 3,
    DtInt8 = 4,
    DtInt16 = 5,
    DtInt32 = 6,
    DtInt64 = 7,
    DtUint8 = 8,
    DtUint16 = 9,
    DtUint32 = 10,
    DtUint64 = 11,
    DtString = 12,
}
impl DataType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DataType::DtInvalid => "DT_INVALID",
            DataType::DtF16 => "DT_F16",
            DataType::DtF32 => "DT_F32",
            DataType::DtF64 => "DT_F64",
            DataType::DtInt8 => "DT_INT8",
            DataType::DtInt16 => "DT_INT16",
            DataType::DtInt32 => "DT_INT32",
            DataType::DtInt64 => "DT_INT64",
            DataType::DtUint8 => "DT_UINT8",
            DataType::DtUint16 => "DT_UINT16",
            DataType::DtUint32 => "DT_UINT32",
            DataType::DtUint64 => "DT_UINT64",
            DataType::DtString => "DT_STRING",
        }
    }
}
/// Protocol buffer representing the shape of tensors.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TensorShape {
    /// Dimensions of the tensor, such as {"input", 30}, {"output", 40}
    /// for a 30 x 40 2D tensor.  If an entry has size -1, this
    /// corresponds to a dimension of unknown size. The names are
    /// optional.
    ///
    /// The order of entries in "dim" matters: It indicates the layout of the
    /// values in the tensor in-memory representation.
    ///
    /// The first entry in "dim" is the outermost dimension used to layout the
    /// values, the last entry is the innermost dimension.
    ///
    /// Dimensions of a tensor.
    #[prost(message, repeated, tag="2")]
    pub dim: ::prost::alloc::vec::Vec<tensor_shape::Dim>,
    /// If true, the number of dimensions in the shape is unknown.
    ///
    /// If true, "dim.size()" must be 0.
    #[prost(bool, tag="3")]
    pub unknown_rank: bool,
}
/// Nested message and enum types in `TensorShape`.
pub mod tensor_shape {
    /// One dimension of the tensor.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Dim {
        /// Size of the tensor in that dimension.
        /// This value must be >= -1, but values of -1 are reserved for "unknown"
        /// shapes (values of -1 mean "unknown" dimension).
        #[prost(int32, tag="1")]
        pub size: i32,
        /// Optional name of the tensor dimension.
        #[prost(string, tag="2")]
        pub name: ::prost::alloc::string::String,
    }
}
/// Protocol Buffer representing a Tensor.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TensorProto {
    /// Stores the data type of the original tensor in order to 
    /// deserialize from raw bytes. 
    #[prost(enumeration="DataType", tag="1")]
    pub tensor_dtype: i32,
    /// Shape of the tensor. Attached as meta data as the tensor gets sent 
    /// as a 1-dim vector. Otherwise reconstructing the model would not be possible
    /// anymore. 
    #[prost(message, optional, tag="2")]
    pub tensor_shape: ::core::option::Option<TensorShape>,
    /// Serialized raw tensor content. This representation
    /// can be used for all tensor types.
    #[prost(bytes="vec", tag="3")]
    pub tensor_content: ::prost::alloc::vec::Vec<u8>,
    // Type specific representations that make it easy to create tensor protos in
    // all languages.  Only the representation corresponding to "dtype" can
    // be set.  The values hold the flattened representation of the tensor in
    // row major order.

    /// DT_F32.
    #[prost(float, repeated, tag="4")]
    pub f32_values: ::prost::alloc::vec::Vec<f32>,
    /// DT_F64.
    #[prost(double, repeated, tag="5")]
    pub f64_values: ::prost::alloc::vec::Vec<f64>,
    /// DT_INT32. DT_INT16, DT_UINT16, DT_INT8, DT_UINT8.
    #[prost(int32, repeated, tag="6")]
    pub i32_values: ::prost::alloc::vec::Vec<i32>,
    /// DT_INT64.
    #[prost(int64, repeated, tag="7")]
    pub i64_values: ::prost::alloc::vec::Vec<i64>,
    /// DT_STRING
    #[prost(bytes="vec", repeated, tag="8")]
    pub string_val: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServerMessage {
    /// `ServerMessage` implements many fields but at most one field 
    /// will be set at the same time.
    ///
    /// Setting any member of the oneof automatically clears all the other members.
    #[prost(oneof="server_message::Msg", tags="1, 2, 3, 4")]
    pub msg: ::core::option::Option<server_message::Msg>,
}
/// Nested message and enum types in `ServerMessage`.
pub mod server_message {
    /// Server distributes the configurations to the clients.
    ///
    /// These configurations contain the necessary information and hyperparameters
    /// that define the Federated Learning process.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SetConfigs {
    }
    /// Reconnect Client
    ///
    /// If a device is not selected for participation, 
    /// the server responds with instructions to reconnect at a later point in time.
    /// 
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ReconnectClient {
        #[prost(int64, tag="1")]
        pub time: i64,
    }
    /// Response to global model request.
    ///
    /// Redistributes the latestes global model.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GlobalModelResponse {
        /// Model represented as a repreated stream of tensors.
        #[prost(message, repeated, tag="1")]
        pub model: ::prost::alloc::vec::Vec<super::TensorProto>,
    }
    /// Placeholder
    ///
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ServerStatus {
        #[prost(string, tag="1")]
        pub status: ::prost::alloc::string::String,
    }
    /// `ServerMessage` implements many fields but at most one field 
    /// will be set at the same time.
    ///
    /// Setting any member of the oneof automatically clears all the other members.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Msg {
        #[prost(message, tag="1")]
        SetConfig(SetConfigs),
        #[prost(message, tag="2")]
        Reconnect(ReconnectClient),
        #[prost(message, tag="3")]
        GlobalModel(GlobalModelResponse),
        #[prost(message, tag="4")]
        Status(ServerStatus),
    }
}
/// Composition of client communication within the msflp protocol 
/// bidirectional stream.
///
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientMessage {
    /// `ClientMessage` implements many fields but at most one field 
    /// will be set at the same time.
    ///
    /// Setting any member of the oneof automatically clears all the other members.
    #[prost(oneof="client_message::Msg", tags="1, 2, 3, 4")]
    pub msg: ::core::option::Option<client_message::Msg>,
}
/// Nested message and enum types in `ClientMessage`.
pub mod client_message {
    /// Disconnect from Stream.
    ///
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ClientDisconnect {
        #[prost(enumeration="super::Reason", tag="1")]
        pub reason: i32,
    }
    /// ClientUpdate providing a model update.
    ///
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ClientUpdate {
        /// Client Identifier that will be sent additionally to a public key.
        ///
        /// Optional as the pk identifies the client but client_id is more
        /// human readable.
        #[prost(uint32, tag="1")]
        pub client_id: u32,
        /// The public key of the client.
        ///
        #[prost(bytes="vec", tag="2")]
        pub client_pk: ::prost::alloc::vec::Vec<u8>,
        /// Seed that is used to mask the model.
        /// 
        /// Is optional as masking the model is optional.
        #[prost(bytes="vec", tag="3")]
        pub local_seed: ::prost::alloc::vec::Vec<u8>,
        /// Model represented as a repreated stream of tensors.
        ///
        /// For more information have a look at `TensorProto`.
        #[prost(message, repeated, tag="4")]
        pub model: ::prost::alloc::vec::Vec<super::TensorProto>,
        /// Model version represents the update round it was trained for.
        ///
        #[prost(uint32, tag="5")]
        pub model_version: u32,
    }
    /// Client request for the latest global model.
    ///
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GlobalModelRequest {
    }
    /// Reporting
    ///
    /// After MSFLP protocol execution, the client reports
    /// computed updates and metrics to the server and cleans up
    /// any temporary resources.
    ///
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ClientReporting {
    }
    /// `ClientMessage` implements many fields but at most one field 
    /// will be set at the same time.
    ///
    /// Setting any member of the oneof automatically clears all the other members.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Msg {
        #[prost(message, tag="1")]
        Disconnect(ClientDisconnect),
        #[prost(message, tag="2")]
        Update(ClientUpdate),
        #[prost(message, tag="3")]
        GlobalModelRequest(GlobalModelRequest),
        #[prost(message, tag="4")]
        Reporting(ClientReporting),
    }
}
/// Reason
///
/// Holds the various reasons when client will be rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Reason {
    /// Unknown reason.
    Null = 0,
}
impl Reason {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Reason::Null => "NULL",
        }
    }
}
/// Generated client implementations.
pub mod msflp_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Mosaic Secure Federated Learning Protocol
    ///
    /// Clients check in to the server by opening a bidirectional stream.
    /// The stream is used to track liveness and orchestrate multi-step communication.
    ///
    #[derive(Debug, Clone)]
    pub struct MsflpClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MsflpClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> MsflpClient<T>
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
        ) -> MsflpClient<InterceptedService<T, F>>
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
            MsflpClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn handle(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::ClientMessage>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::ServerMessage>>,
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
            let path = http::uri::PathAndQuery::from_static(
                "/mosaic.protos.MSFLP/Handle",
            );
            self.inner.streaming(request.into_streaming_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod msflp_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with MsflpServer.
    #[async_trait]
    pub trait Msflp: Send + Sync + 'static {
        ///Server streaming response type for the Handle method.
        type HandleStream: futures_core::Stream<
                Item = Result<super::ServerMessage, tonic::Status>,
            >
            + Send
            + 'static;
        async fn handle(
            &self,
            request: tonic::Request<tonic::Streaming<super::ClientMessage>>,
        ) -> Result<tonic::Response<Self::HandleStream>, tonic::Status>;
    }
    /// Mosaic Secure Federated Learning Protocol
    ///
    /// Clients check in to the server by opening a bidirectional stream.
    /// The stream is used to track liveness and orchestrate multi-step communication.
    ///
    #[derive(Debug)]
    pub struct MsflpServer<T: Msflp> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Msflp> MsflpServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
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
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MsflpServer<T>
    where
        T: Msflp,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/mosaic.protos.MSFLP/Handle" => {
                    #[allow(non_camel_case_types)]
                    struct HandleSvc<T: Msflp>(pub Arc<T>);
                    impl<T: Msflp> tonic::server::StreamingService<super::ClientMessage>
                    for HandleSvc<T> {
                        type Response = super::ServerMessage;
                        type ResponseStream = T::HandleStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                tonic::Streaming<super::ClientMessage>,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).handle(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = HandleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.streaming(method, req).await;
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
    impl<T: Msflp> Clone for MsflpServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Msflp> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Msflp> tonic::server::NamedService for MsflpServer<T> {
        const NAME: &'static str = "mosaic.protos.MSFLP";
    }
}
