pub mod fetcher;

// #[derive(Clone)]
// pub struct MessageHandler {
//     parser: Parser,
// }
//
// impl MessageHandler {
//     pub fn new() -> Self {
//         let parser = Parser::new();
//
//         Self { parser }
//     }
// }
//
// struct RawMessage {
//     /// The buffer that contains the message to parse
//     message: Vec<f64>,
// }
//
// impl Clone for RawMessage {
//     fn clone(&self) -> Self {
//         Self {
//             message: self.message.clone(),
//         }
//     }
// }

// #[derive(Debug, Clone)]
// struct Parser;
//
// impl Service<RawMessage> for Parser {
//     type Response = Message;
//     type Error = Infallible;
//     type Future = future::Ready<Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Poll::Ready(Ok(()))
//     }
//
//     fn call(&mut self, req: RawMessage) -> Self::Future {
//         let bytes = req.message.inner();
//         future::ready(&bytes)
//     }
// }

// impl<T> From<MessageBuffer<T>> for RawMessage<T> {
//     fn from(message: Vec<f64>) -> Self {
//         RawMessage {
//             message: Arc::new(message),
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct MessageParser(inner);
//
// impl<T> Service<T> for MessageParser
// where
//     T: AsRef<[u8]> + Sync + Send + 'static,
// {
//     type Response = Message;
//     type Error = Infallible;
//     type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         <inner as Service<T>>::poll_ready(&mut self.0, cx)
//     }
//
//     fn call(&mut self, req: T) -> Self::Future {
//         let fut = self.0.call(req);
//         Box::pin(async move { fut.await })
//     }
// }
