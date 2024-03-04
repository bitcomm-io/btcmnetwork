use std::error::Error;

use crate::{
    eventqueue::{MessageEvent, MessageEventQueue},
    group::grpmessage,
    sendtoclient,
};

/// 启动消息事件队列服务器。
/// 
/// 该函数从消息事件队列中获取事件并处理，根据事件类型分发到相应的处理函数中。
/// 
/// # 返回
/// 如果启动成功，则返回 `Ok(())`，否则返回包含错误信息的 `Result`。
#[allow(unused_variables)]
pub async fn start_message_event_queue_server() -> Result<(), Box<dyn Error>> {
    // 获取消息事件队列的接收端
    let receiver = MessageEventQueue::get_receiver();
    let mut meqrece = receiver.lock().await;
    while let Some(event) = meqrece.recv().await {
        match event {
            // 处理消息接收事件
            MessageEvent::MessageReceive { reqmsgbuff, reqmsggram } => {
                sendtoclient::send_message_to_client(&reqmsgbuff, &reqmsggram).await;
            }
            // 处理群组消息接收事件
            MessageEvent::GroupReceive { reqmsgbuff, reqmsggram } => {
                grpmessage::send_group_message_to_all(&reqmsgbuff, &reqmsggram).await;
            }
            // 处理服务消息接收事件
            MessageEvent::ServiceReceive { reqmsgbuff, reqmsggram } => {
                // 可根据需要添加处理逻辑
            }
            _ => {} // 忽略其他类型的事件
        }
    }
    Ok(())
}

// use std:: error::Error ;

// use crate::{
//     eventqueue::{MessageEvent, MessageEventQueue},
//     group::grpmessage,
//     sendtoclient,
// };

// #[allow(unused_variables)]
// pub async fn start_message_event_queue_server() -> Result<(), Box<dyn Error>> {
//     let receiver = MessageEventQueue::get_receiver();
//     let mut meqrece = receiver.lock().await;//MESSAGE_CHANNEL.1.lock().await;
//     while let Some(event) = meqrece.recv().await {
//         match event {
//             MessageEvent::MessageReceive { reqmsgbuff, reqmsggram } => {
//                 sendtoclient::send_message_to_client( &reqmsgbuff, &reqmsggram).await;
//             }
//             MessageEvent::GroupReceive { reqmsgbuff, reqmsggram } => {
//                 // 处理 GroupReceive 事件
//                 grpmessage::send_group_message_to_all( &reqmsgbuff, &reqmsggram).await;
//             }
//             MessageEvent::ServiceReceive { reqmsgbuff, reqmsggram } => {
//                 // 处理 ServiceReceive 事件
//             }
//             _ => {}
//         }
//     }
//     Ok(())
// }
