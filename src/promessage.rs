use btcmbase::datagram::MessageDataGram;
use bytes::Bytes;
#[allow(unused_imports)]
use s2n_quic::{
    stream::{BidirectionalStream, SendStream},
    Server,
};
use tokio::sync::{mpsc::Sender, Mutex};

use std::sync::Arc;
// use tokio::sync::Mutex;
use crate::{connservice::ClientPoolManager, eventqueue::MessageEvent, send};

#[allow(unused_variables)]
pub fn handle_message_data(reqmsgbuff: &Arc<Bytes>, reqmsggram: &Arc<MessageDataGram>) {}

#[allow(unused_variables)]
pub async fn process_message_data<'a>(
    stmid: u64,
    reqmsgbuff: &Arc<Bytes>,
    reqmsggram: &Arc<MessageDataGram>,
    cpm: Arc<tokio::sync::Mutex<ClientPoolManager>>,
    stm: Arc<tokio::sync::Mutex<SendStream>>,
    meqsend: Arc<Mutex<Sender<MessageEvent>>>,
) {
    send::send_message(stmid, reqmsgbuff, reqmsggram, cpm, stm, meqsend).await;
}
