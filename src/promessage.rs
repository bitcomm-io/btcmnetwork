use btcmbase::datagram::MessageDataGram;
use bytes::Bytes;
use s2n_quic::stream::SendStream;

use std::sync::Arc;
use crate::recefromclient;

/// 处理消息数据
///
/// 此函数用于处理接收到的消息数据，但不执行任何操作。
///
/// # Arguments
///
/// * `reqmsgbuff` - 消息缓冲区的 Arc<Bytes> 引用
/// * `reqmsggram` - 消息数据报的 Arc<MessageDataGram> 引用
///
/// # Examples
///
/// ```rust
/// use btcmbase::datagram::MessageDataGram;
/// use bytes::Bytes;
/// use std::sync::Arc;
/// use crate::handle_message_data;
///
/// let reqmsgbuff = Arc::new(Bytes::new());
/// let reqmsggram = Arc::new(MessageDataGram::default());
/// handle_message_data(&reqmsgbuff, &reqmsggram);
/// ```
#[allow(unused_variables)]
pub fn handle_message_data(reqmsgbuff: &Arc<Bytes>, reqmsggram: &Arc<MessageDataGram>) {}

/// 异步处理消息数据
///
/// 此函数用于异步处理接收到的消息数据，将消息数据转发给接收消息的客户端。
///
/// # Arguments
///
/// * `reqmsgbuff` - 消息缓冲区的 Arc<Bytes> 引用
/// * `reqmsggram` - 消息数据报的 Arc<MessageDataGram> 引用
/// * `stm` - 发送流的互斥锁的 Arc<tokio::sync::Mutex<SendStream>> 引用
///
/// # Examples
///
/// ```rust
/// use btcmbase::datagram::MessageDataGram;
/// use bytes::Bytes;
/// use std::sync::Arc;
/// use s2n_quic::stream::SendStream;
/// use tokio::sync::Mutex;
/// use crate::process_message_data;
///
/// let reqmsgbuff = Arc::new(Bytes::new());
/// let reqmsggram = Arc::new(MessageDataGram::default());
/// let stm = Arc::new(Mutex::new(SendStream::default()));
/// tokio::spawn(async move {
///     process_message_data(&reqmsgbuff, &reqmsggram, stm).await;
/// });
/// ```
#[allow(unused_variables)]
pub async fn process_message_data<'a>(
    reqmsgbuff: &Arc<Bytes>,
    reqmsggram: &Arc<MessageDataGram>,
    stm: Arc<tokio::sync::Mutex<SendStream>>
) {
    recefromclient::rece_message_from_client(reqmsgbuff, reqmsggram, stm).await;
}

// use btcmbase::datagram::MessageDataGram;
// use bytes::Bytes;
// #[allow(unused_imports)]
// use s2n_quic::{ stream::{ BidirectionalStream, SendStream }, Server };
// // use tokio::sync::{mpsc::Sender, Mutex};

// use std::sync::Arc;
// // use tokio::sync::Mutex;
// use crate::recefromclient;

// #[allow(unused_variables)]
// pub fn handle_message_data(reqmsgbuff: &Arc<Bytes>, reqmsggram: &Arc<MessageDataGram>) {}

// #[allow(unused_variables)]
// pub async fn process_message_data<'a>(
//     reqmsgbuff: &Arc<Bytes>,
//     reqmsggram: &Arc<MessageDataGram>,
//     stm: Arc<tokio::sync::Mutex<SendStream>>
// ) {
//     recefromclient::rece_message_from_client(reqmsgbuff, reqmsggram, stm).await;
// }
