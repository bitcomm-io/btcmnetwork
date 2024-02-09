use std::sync::Arc;

use btcmbase::datagram::MessageDataGram;
use bytes::Bytes;
// use btcmbase::datagram::InnerDataGram;
// use getset::{Getters, Setters};
use tokio::sync::mpsc;



#[warn(dead_code)]
pub static EVENT_QUEUE_LEN: usize = 1024;

// 定义事件类型
#[warn(dead_code)]
pub enum MessageEvent {
    MessageReceive {reqmsgbuff:Arc<Bytes>,reqmsggram:Arc<MessageDataGram>} 
}
#[allow(dead_code)]
#[derive(Debug)]//Getters, Setters)]
pub struct MessageEventQueue {
    // 发送端
    // #[getset(set = "pub", get = "pub")]
    pub sender   : mpsc::Sender<MessageEvent>,   // 发送端
    // 接收端
    // #[getset(set = "pub", get = "pub")]
    pub receiver : mpsc::Receiver<MessageEvent>, // 接收端
}

#[warn(dead_code)]
impl MessageEventQueue {
    // 
    pub fn new() -> Self {
        // 创建一个 mpsc::channel，消息类型为MessageEvent
        let (sender, receiver) = mpsc::channel::<MessageEvent>(EVENT_QUEUE_LEN);
        // 返回一个新的结构体实例
        Self { sender, receiver }    
    }

}