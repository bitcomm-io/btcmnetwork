use std::sync::Arc;

use btcmbase::datagram::BitcommFlag;
// use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::io::AsyncWriteExt;

use crate::connservice::{self, ClientPoolManager};
// 
// use btcmbase::datagram::CommandDataGram;
// use bytes::Bytes;
// use s2n_quic::stream::SendStream;
// #[allow(unused_imports)]
// use tokio::io::AsyncWriteExt;

// use crate::connservice::ClientPoolManager;
// use tokio::sync::{mpsc::Sender, Mutex};

// use crate::eventqueue::MessageEvent;

// 登出代码
const PINGPONG_CODE: u32 = 0x000A;


#[allow(unused_variables)]
pub async fn process_bitcomm_flag_data<'a>(
    pingpong: &Arc<BitcommFlag>,
    stm: Arc<tokio::sync::Mutex<SendStream>>,
) {
    let mut stream = stm.lock().await;
    if let Ok(address) = stream.connection().remote_addr() {
        let address_string = address.to_string();
        if let Some(key) = ClientPoolManager::get_addres(&address_string).await {
            let (client, device) = connservice::get_cd_by_key(key);
            if let Some(ostm) = ClientPoolManager::get_client(client, device).await {
                // 如果是指向同一个Stream
                if Arc::ptr_eq(&stm, &ostm) {
                    // 更新客户端信息
                    ClientPoolManager::update_client(client, device).await;
                    // 返回PONG
                    // stream.write_u32_le(n)
                    stream.write_u32_le(BitcommFlag::BITCOMM_PONG.bits()).await.unwrap();
                    stream.flush().await.unwrap();
                } else { // 如果不是同一个链接,这两个Stream都需要关闭
                    ClientPoolManager::remove_addres(&address_string).await;
                    ClientPoolManager::remove_client(client, device).await;
                    stream.close().await.unwrap();
                    stream.connection().close(PINGPONG_CODE.into());
                    let mut os = ostm.lock().await;
                    os.close().await.unwrap();
                    os.connection().close(PINGPONG_CODE.into());
                }
            }
        }
    }
}

