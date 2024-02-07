use std::sync::Arc;

use btcmbase::datagram::{CommandDataGram, MessageDataGram};
use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::io::AsyncWriteExt;

use crate::connservice::ClientPoolManager;



#[allow(unused_variables)]
// #[allow(dead_code)]
pub async fn send_message_to_client<'a>(stmid   :u64,
                                reqmsgbuff:&Arc<Bytes>,reqmsggram:&Arc<MessageDataGram>,
                                cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                                stm     :Arc<tokio::sync::Mutex<SendStream>>) {
    eprintln!("client send message buf  to server {:?}", reqmsgbuff);  
    eprintln!("client send message gram to server {:?}", reqmsggram);  

    // let command = data.req_cmdgram.unwrap();
        // 回复已送达信息
        let mut vecu8 = CommandDataGram::create_gram_buf(0);
        let cdg = CommandDataGram::create_command_gram_from_message_gram(vecu8.as_mut(), reqmsggram.as_ref());
        let mut stream = stm.lock().await;
        // let mut bts = rescmdbuff.as_ref();
        // let u8array = vecu8.as_mut();
        stream.write_all(vecu8.as_mut()).await.expect("stream should be open");
        stream.flush().await.expect("stream should be open");

    //转发给接收者的不同设备
        let ccp = cpm.lock().await;
        // 如何能够获取hash
        if let Some(devhash) = ccp.get_client(reqmsggram.receiver().into()) {
            // 循环处理每一个键值对(每一个设备，对应一个连接)
            for (key, value) in devhash.iter() {
                println!("Key: {}, Value: {}", key, value);
                if let Some(ostm) = ccp.get_stream(*value) {
                    let mut ostream = ostm.lock().await;
                    // let byte_slice_ref: &[u8] = reqmsgbuff;
                    // let ref = reqmsgbuff.bytes();
                    // ostream.send(reqmsgbuff.bytes());
                    ostream.write_all(reqmsgbuff).await.expect("send to error!");
                    ostream.flush().await.expect("flush error");
                }
            }
        }
}