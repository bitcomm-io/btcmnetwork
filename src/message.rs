use std::sync::Arc;

use btcmbase::datagram::MessageDataGram;
use bytes::Bytes;
use s2n_quic::stream::SendStream;

use crate::connservice::ClientPoolManager;



#[allow(unused_variables)]
#[allow(dead_code)]
pub async fn send_message_to_client<'a>(stmid   :u64,
                                reqmsgbuff:&Arc<Bytes>,reqmsggram:&Arc<MessageDataGram>,
                                cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                                stm     :Arc<tokio::sync::Mutex<SendStream>>) {
    eprintln!("client send message buf  to server {:?}", reqmsgbuff);  
    eprintln!("client send message gram to server {:?}", reqmsggram);  
}