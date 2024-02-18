use std::sync::Arc;

use btcmbase::datagram::BitcommFlag;
// use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::sync::{mpsc::Sender, Mutex};

use crate::{connservice::ClientPoolManager, eventqueue::MessageEvent};



#[allow(unused_variables)]
pub async fn process_bitcomm_flag_data<'a>(
    stmid: u64,
    pingpong: &Arc<BitcommFlag>,
    cpm: Arc<tokio::sync::Mutex<ClientPoolManager>>,
    stm: Arc<tokio::sync::Mutex<SendStream>>,
    meqsend: Arc<Mutex<Sender<MessageEvent>>>,
) {

}
