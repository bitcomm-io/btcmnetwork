use btcmbase::client::DeviceConnState;
use bytes::Bytes;
use std::{error::Error, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    sync::{mpsc::Receiver, Mutex},
};

use crate::{connservice::ClientPoolManager, eventqueue::MessageEvent};

pub async fn start_message_evnet_queue_server(
    cpm: Arc<tokio::sync::Mutex<ClientPoolManager>>,
    meqrece0: Arc<Mutex<Receiver<MessageEvent>>>,
) -> Result<(), Box<dyn Error>> {
    let mut meqrece1 = meqrece0.lock().await;
    // 处理接收到的事件
    while let Some(event) = meqrece1.recv().await {
        match event {
            #[allow(unused_variables)]
            MessageEvent::MessageReceive {
                reqmsgbuff,
                reqmsggram,
            } => {
                let receiver = reqmsggram.receiver();
                let ccp = cpm.lock().await;
                // 如果能够获取Vec,是登录的同一个服务器,则在服务器内部传递消息
                if let Some(devhash) = ccp.get_device_pool(receiver.into()) {
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
                                send_request_online(&ccp, receiver.into(), *did, &reqmsgbuff).await;
                            }
                            _ => { // 其他
                            }
                        }
                    }
                } else { // 如果获取不到,说明不是登录的同一个服务器,或是处在离线状态,则需要通过nats进行消息传递
                }
            }
        }
    }
    Ok(())
}
fn send_request_offline() {}
fn send_request_onback() {}
//
async fn send_request_online(ccp: &ClientPoolManager, clt: u64, did: u32, reqmsgbuff: &Bytes) {
    let value = ccp.get_client(clt, did).unwrap();
    let mut stream = value.lock().await;
    stream.write_all(reqmsgbuff).await.expect("send to error!");
    stream.flush().await.expect("flush error");
}
