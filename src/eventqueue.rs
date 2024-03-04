use std::sync::Arc;

use btcmbase::datagram::MessageDataGram;
use bytes::Bytes;
use tokio::sync::{mpsc::{self, Receiver, Sender}, Mutex};

/// 默认事件队列长度。
#[warn(dead_code)]
pub static EVENT_QUEUE_LEN: usize = 1024;

/// 事件类型枚举。
#[warn(dead_code)]
pub enum MessageEvent {
    MessageReceive {
        reqmsgbuff: Arc<Bytes>,
        reqmsggram: Arc<MessageDataGram>,
    },
    GroupReceive {
        reqmsgbuff: Arc<Bytes>,
        reqmsggram: Arc<MessageDataGram>,
    },
    ServiceReceive {
        reqmsgbuff: Arc<Bytes>,
        reqmsggram: Arc<MessageDataGram>,
    },
    DeviceReceive {
        reqmsgbuff: Arc<Bytes>,
        reqmsggram: Arc<MessageDataGram>,
    },
    RobotReceive {
        reqmsgbuff: Arc<Bytes>,
        reqmsggram: Arc<MessageDataGram>,
    },
}

/// 消息事件队列结构。
#[allow(dead_code)]
#[derive(Debug)] //Getters, Setters)]
pub struct MessageEventQueue {
    // // 发送端
    // // #[getset(set = "pub", get = "pub")]
    // pub sender: Arc<Mutex<mpsc::Sender<MessageEvent>>>, // 发送端
    // // 接收端
    // // #[getset(set = "pub", get = "pub")]
    // pub receiver: Arc<Mutex<mpsc::Receiver<MessageEvent>>>, // 接收端
}

use lazy_static::lazy_static;
lazy_static! {  
    /// 全局消息通道元组,用于处理客户端发送来的所有消息
    static ref MESSAGE_CHANNEL : (Arc<Mutex<Sender<MessageEvent>>>, Arc<Mutex<Receiver<MessageEvent>>>) =  {
        let (sender, receiver) = mpsc::channel::<MessageEvent>(EVENT_QUEUE_LEN);
        (Arc::new(Mutex::new(sender)), Arc::new(Mutex::new(receiver)) )
    };
}

impl MessageEventQueue {
    /// 获取消息发送端。
    pub fn get_sender(
    ) -> Arc<Mutex<Sender<MessageEvent>>> {
        MESSAGE_CHANNEL.0.clone()
    }
    /// 获取消息接收端。
    pub fn get_receiver(
    ) -> Arc<Mutex<Receiver<MessageEvent>>> {
        MESSAGE_CHANNEL.1.clone()
    }
}

