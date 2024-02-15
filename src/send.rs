use std::sync::Arc;

use btcmbase::datagram::{CommandDataGram, MessageDataGram, ReturnCode};
use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::{io::AsyncWriteExt, sync::{mpsc::Sender, Mutex}};

use crate::{connservice::ClientPoolManager, eventqueue::MessageEvent};



#[allow(unused_variables)]
// #[allow(dead_code)]
pub async fn send_message<'a>(stmid   :u64,
                                reqmsgbuff:&Arc<Bytes>,reqmsggram:&Arc<MessageDataGram>,
                                cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                                stm     :Arc<tokio::sync::Mutex<SendStream>>,
                                meqsend :Arc<Mutex<Sender<MessageEvent>>>) ->Option<Arc<MessageDataGram>> {
    // 
    // eprintln!("client send message buf  to server {:?}", reqmsgbuff);  
    slog::info!(btcmtools::LOGGER,"client send message gram to server {:?}", reqmsggram);  
    // 
    let mut ccp = cpm.lock().await;
    // 
    let clt:u64  = reqmsggram.sender().into(); let dev:u32  = reqmsggram.deviceid();
    // 如果能够获取stream
    if let Some(ostm) = ccp.get_client(clt, dev) {
        // 并且pool里的和传递过来的是同一个stream
        if Arc::ptr_eq(ostm, &stm) {
            // 回复已送达信息
            send_message_receipt(reqmsggram, stm).await;
            // 将消息发送到事件队列中
            send_message_to_queue(reqmsgbuff,reqmsggram,meqsend).await;
            // 更新client的超时信息
            ccp.update_client(clt, dev);
        } else {
            // 出错,返回出错的回执信息(两个连接不一致)
            send_message_twosession_nosame(reqmsggram, stm).await;
        }
    } else {
        // 出错,返回出错的回执信息(client_pool中没有连接,说明没有登录或是已超时被删除)
        send_message_notlogin(reqmsggram, stm).await;
    }
    Option::Some(reqmsggram.clone())
}
// 
async fn send_message_notlogin(reqmsggram: &Arc<MessageDataGram>, stm: Arc<Mutex<SendStream>>) {
    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let cdg = CommandDataGram::create_command_gram_from_message_gram(vecu8.as_mut(), reqmsggram.as_ref());
    cdg.set_returncode(ReturnCode::RETURN_NOT_LOGIN);
    let mut stream = stm.lock().await;
    // let mut bts = rescmdbuff.as_ref();
    // let u8array = vecu8.as_mut();
    stream.write_all(vecu8.as_mut()).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}
// 
async fn send_message_twosession_nosame(reqmsggram: &Arc<MessageDataGram>, stm: Arc<Mutex<SendStream>>) {
    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let cdg = CommandDataGram::create_command_gram_from_message_gram(vecu8.as_mut(), reqmsggram.as_ref());
    cdg.set_returncode(ReturnCode::RETURN_TWO_SESSION_NO_SAME);
    let mut stream = stm.lock().await;
    // let mut bts = rescmdbuff.as_ref();
    // let u8array = vecu8.as_mut();
    stream.write_all(vecu8.as_mut()).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}
// 
async fn send_message_receipt(reqmsggram: &Arc<MessageDataGram>, stm: Arc<Mutex<SendStream>>) {
    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let cdg = CommandDataGram::create_command_gram_from_message_gram(vecu8.as_mut(), reqmsggram.as_ref());
    cdg.set_returncode(ReturnCode::RETURN_OK);
    let mut stream = stm.lock().await;
    // let mut bts = rescmdbuff.as_ref();
    // let u8array = vecu8.as_mut();
    stream.write_all(vecu8.as_mut()).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}
// 
async fn send_message_to_queue(reqmsgbuff  :&Arc<Bytes>,
                               reqmsggram  :&Arc<MessageDataGram>,
                               meqsend     :Arc<Mutex<Sender<MessageEvent>>>) {

    let msgevent = meqsend.lock().await;
    msgevent.send(MessageEvent::MessageReceive { reqmsgbuff:reqmsgbuff.clone(), reqmsggram: reqmsggram.clone() }).await.expect("send event error!");
}