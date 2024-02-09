use std::sync::Arc;

use btcmbase::datagram::{CommandDataGram, MessageDataGram};
use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::{io::AsyncWriteExt, sync::{mpsc::Sender, Mutex}};

use crate::{connservice::ClientPoolManager, eventqueue::MessageEvent};



#[allow(unused_variables)]
// #[allow(dead_code)]
pub async fn send_message_to_return<'a>(stmid   :u64,
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

}

#[allow(unused_variables)]
// #[allow(dead_code)]
pub async fn send_message_to_queue(reqmsgbuff  :&Arc<Bytes>,
                                    reqmsggram  :&Arc<MessageDataGram>,
                                    meqsend     :Arc<Mutex<Sender<MessageEvent>>>) {

    let msgevent = meqsend.lock().await;
    msgevent.send(MessageEvent::MessageReceive { reqmsgbuff:reqmsgbuff.clone(), reqmsggram: reqmsggram.clone() }).await.expect("send event error!");
    //转发给接收者的不同设备
        // let sender = reqmsggram.sender();
        // let receiver = reqmsggram.receiver();
        // let deviceid = reqmsggram.deviceid();
    
        // let ccp = cpm.lock().await;
        // // 如何能够获取hash
        // if let Some(devhash) = ccp.get_client(receiver.into()) {
        //     // 循环处理每一个键值对(每一个设备，对应一个连接)
        //     for (key, value) in devhash.iter() {
        //         // println!("Key: {}, Value: {}", key, value);
        //         // 如果指向同一个流
        //         if Arc::ptr_eq(&stm, value) {
        //             stream.write_all(reqmsgbuff).await.expect("send to error!");
        //             stream.flush().await.expect("flush error");   
        //         } else {
        //             let mut ostream = value.lock().await;
        //             ostream.write_all(reqmsgbuff).await.expect("send to error!");
        //             ostream.flush().await.expect("flush error");
        //         }
        //     }
        // }
        // mqserver::send_message_event(Arc::new(ccp),Arc::new(stream),meq,reqmsgbuff.clone(), reqmsggram.clone()).await.unwrap();
}