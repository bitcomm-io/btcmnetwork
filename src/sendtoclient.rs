use btcmbase::{ client::DeviceConnState, datagram::MessageDataGram };
use bytes::Bytes;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use crate::connservice::ClientPoolManager;
use tracing::info;
// use tracing::error;

pub async fn send_message_to_client(
    reqmsgbuff: &Arc<Bytes>,
    reqmsggram: &Arc<MessageDataGram>
) {
    info!("MQ gram to event {:?}", reqmsggram);
    let receiver = reqmsggram.receiver();
    // let ccp = cpm.lock().await;
    // 如果能够获取Vec,是登录的同一个服务器,则在服务器内部传递消息
    if let Some(devhash) = ClientPoolManager::get_device_pool(receiver.into()).await {
        // 循环处理每一个键值对(每一个设备，对应一个连接)
        for (did, dci) in devhash {
            //
            match dci.device_state() {
                DeviceConnState::STATE_OFFLINE => {
                    // 离线
                    send_request_offline();
                }
                DeviceConnState::STATE_ONBACK => {
                    // 后台
                    send_request_onback();
                }
                DeviceConnState::STATE_ONLINE => {
                    // 在线
                    send_request_online(receiver.into(), did, &reqmsgbuff).await;
                }
                _ => {
                    // 其他
                }
            }
        }
    } else {
        // 如果获取不到,说明不是登录的同一个服务器,或是处在离线状态,则需要通过nats进行消息传递
    }
}

fn send_request_offline() {}
fn send_request_onback() {}
//
async fn send_request_online(clt: u64, did: u32, reqmsgbuff: &Bytes) {
    if let Some(value) = ClientPoolManager::get_client(clt, did).await {
        let mut stream = value.lock().await;
        stream.write_all(reqmsgbuff).await.expect("send to error!");
        stream.flush().await.expect("flush error");
    }
}
