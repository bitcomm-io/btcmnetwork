use std::{error::Error, sync::Arc};
#[allow(unused_imports)]
use btcmbase::datagram::MessageDataGram;
#[allow(unused_imports)]
use bytes::Bytes;
#[allow(unused_imports)]
use s2n_quic::stream::SendStream;
use tokio::{io::AsyncWriteExt, sync::{mpsc::Receiver, Mutex}};

use crate::{connservice::ClientPoolManager, eventqueue::MessageEvent};



pub async fn start_message_evnet_queue_server(cpm:Arc<tokio::sync::Mutex<ClientPoolManager>>,
                                              meqrece0:Arc<Mutex<Receiver<MessageEvent>>>) 
                                              -> Result<(), Box<dyn Error>> {

    let mut meqrece1 = meqrece0.lock().await;
    // cpm.
    // 处理接收到的事件
    while let Some(event) = meqrece1.recv().await {
        match event {
            #[allow(unused_variables)]
            MessageEvent::MessageReceive{ reqmsgbuff, reqmsggram } => {
                eprintln!("MQ buf  to server {:?}", reqmsgbuff);  
                eprintln!("MQ gram to server {:?}", reqmsggram);  
                let sender = reqmsggram.sender();
                let receiver = reqmsggram.receiver();
                let deviceid = reqmsggram.deviceid();
                let ccp = cpm.lock().await;
                // 如何能够获取hash
                if let Some(devhash) = ccp.get_client(receiver.into()) {
                    // 循环处理每一个键值对(每一个设备，对应一个连接)
                    for (key, value) in devhash.iter() {
                        // println!("Key: {}, Value: {}", key, value);
                        let mut stream = value.lock().await;
                        stream.write_all(&reqmsgbuff).await.expect("send to error!");
                        stream.flush().await.expect("flush error");
                    }
                }
            }
        }
    }
    Ok(())
}