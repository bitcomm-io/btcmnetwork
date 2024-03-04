use std::{ error::Error, sync::Arc };

use btcmbase::{ client::ClientID, datagram::MessageDataGram, group::ClientGroup };
use bytes::Bytes;
use tokio::sync::{ mpsc, Mutex };

use crate::group::GroupMessageProcess;

use super::{ GroupMessageEvent, GROUP_MSG_PROCESS_HASH };

/// 向所有群组成员发送群组消息。
///
/// # 参数
/// - `reqmsgbuff`: 请求消息的字节数据。
/// - `reqmsggram`: 请求消息的数据报。
pub async fn send_group_message_to_all(
    reqmsgbuff: &Arc<Bytes>, 
    reqmsggram: &Arc<MessageDataGram>
) {
    // slog::info!(btcmtools::LOGGER, "process Group Message {:?}", reqmsggram);
    // 获取组ID
    let group_id = reqmsggram.receiver();
    if let Some(gmp) = get_gmp_by_id(group_id).await {
        // 成功获取群组
        let sender = gmp.sender.lock().await;
        sender
            .send(GroupMessageEvent::MessageReceive {
                reqmsgbuff: reqmsgbuff.clone(),
                reqmsggram: reqmsggram.clone(),
            }).await
            .unwrap();
    }
}

/// 启动群组消息事件队列服务器。
///
/// # 参数
/// - `client_group`: 客户端群组。
/// - `receiver`: 接收群组消息事件的接收器。
///
/// # 返回
/// 返回结果，如果成功返回`Ok(())`，否则返回错误。
#[allow(unused_variables)]
pub async fn start_group_message_event_queue_server(
    client_group: Arc<ClientGroup>,
    receiver: Arc<Mutex<mpsc::Receiver<GroupMessageEvent>>>
) -> Result<(), Box<dyn Error>> {
    let mut meqrece = receiver.lock().await;
    while let Some(event) = meqrece.recv().await {
        match event {
            GroupMessageEvent::MessageReceive { reqmsgbuff, reqmsggram } => {
                //
                let client_list = client_group.client_list();
                //
                for cltid in client_list {
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// 根据群组ID获取群组消息处理器。
///
/// # 参数
/// - `group_id`: 群组ID。
///
/// # 返回
/// 返回可选的群组消息处理器。
async fn get_gmp_by_id(group_id: ClientID) -> Option<Arc<GroupMessageProcess>> {
    let client_id = group_id.into();
    let mut hash_map = GROUP_MSG_PROCESS_HASH.lock().await;
    // 如何已经存,则直接取出并返回
    if hash_map.contains_key(&client_id) {
        hash_map.get(&client_id).map(|x| x.clone())
    } else {
        // 如果不存在,则需要新建
        // 建群组
        let client_group = get_group_by_id(group_id).await.unwrap();
        // 建群组处理器
        let gmp = Arc::new(GroupMessageProcess::new(client_group).await);
        // let gsh = get_group_message_handle(gmp.clone());
        hash_map.insert(client_id, gmp)
    }
}

/// 根据群组ID获取群组。
///
/// # 参数
/// - `group_id`: 群组ID。
///
/// # 返回
/// 返回可选的群组。
async fn get_group_by_id(group_id: ClientID) -> Option<Arc<ClientGroup>> {
    // 此处需要从Redis或其他Database中加载群组内容
    Some(Arc::new(ClientGroup::new(group_id)))
}
