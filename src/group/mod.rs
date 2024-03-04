use std::{ collections::HashMap, sync::Arc };

use btcmbase::{ datagram::{ CommandDataGram, MessageDataGram }, group::ClientGroup };
use bytes::Bytes;
use getset::{ CopyGetters, Setters };
use tokio::sync::{ mpsc, Mutex };

// use crate::connservice::ClientPoolManager;

pub mod grpmessage;

use lazy_static::lazy_static;
lazy_static! {
    pub static ref GROUP_MSG_PROCESS_HASH: Mutex<HashMap<u64, Arc<GroupMessageProcess>>> = Mutex::new(HashMap::new());
}

#[allow(dead_code)]
pub enum GroupMessageEvent {
    MessageReceive {
        reqmsgbuff: Arc<Bytes>,
        reqmsggram: Arc<MessageDataGram>,
    },
    CommandReceiver {
        reqmsgbuff: Arc<Bytes>,
        reqmsggram: Arc<CommandDataGram>,
    },
}

static GROUP_EVENT_QUEUE_LEN: usize = 1024;

#[repr(C)] // 与C语言兼容
#[derive(Debug, CopyGetters, Setters)]
//
pub struct GroupMessageProcess {
    // 群组
    client_group: Arc<ClientGroup>,
    // 发送端
    #[getset(get = "pub")]
    sender: Arc<Mutex<mpsc::Sender<GroupMessageEvent>>>, // 发送端
    // 接收端
    #[getset(get = "pub")]
    receiver: Arc<Mutex<mpsc::Receiver<GroupMessageEvent>>>, // 接收端
    // 异步处理任务
    #[getset(get = "pub")]
    gmp_handle: Arc<tokio::task::JoinHandle<()>>, //Option<Arc<Mutex<tokio::task::JoinHandle<()>>>>,
}

//
impl GroupMessageProcess {
    //
    pub async fn new(client_group: Arc<ClientGroup>) -> GroupMessageProcess {
        let (sender, receiver) = mpsc::channel::<GroupMessageEvent>(GROUP_EVENT_QUEUE_LEN);
        let sender = Arc::new(Mutex::new(sender));
        let receiver = Arc::new(Mutex::new(receiver));
        let gmp_handle = get_group_message_handle(&client_group, &receiver);
        GroupMessageProcess {
            client_group,
            sender,
            receiver,
            gmp_handle,
        }
    }
}

//
fn get_group_message_handle(
    client_group: &Arc<ClientGroup>,
    receiver: &Arc<Mutex<mpsc::Receiver<GroupMessageEvent>>>
) -> Arc<tokio::task::JoinHandle<()>> {
    let group_server_handle = {
        let cg = client_group.clone();
        let rv = receiver.clone();
        tokio::spawn(async move {
            grpmessage::start_group_message_event_queue_server(cg, rv).await.unwrap();
        })
    };
    Arc::new(group_server_handle)
}
