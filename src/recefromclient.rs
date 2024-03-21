use std::sync::Arc;

use btcmbase::{ client::ClientType, datagram::{ CommandDataGram, MessageDataGram, ReturnCode } };
use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::{ io::AsyncWriteExt, sync::Mutex };
use crate::{ connservice::ClientPoolManager, eventqueue::{ MessageEvent, MessageEventQueue } };
use tracing::info;
// use tracing::error;
#[allow(unused_variables)]
// #[allow(dead_code)]
pub async fn rece_message_from_client(
    reqmsgbuff: &Arc<Bytes>,
    reqmsggram: &Arc<MessageDataGram>,
    stm: Arc<tokio::sync::Mutex<SendStream>>
) -> Option<Arc<MessageDataGram>> {
    //
    // eprintln!("client send message buf  to server {:?}", reqmsgbuff);
    info!("client send message gram to server {:?}", reqmsggram);
    //
    // let mut ccp = cpm.lock().await;
    //
    let clt: u64 = reqmsggram.sender().into();
    let dev: u32 = reqmsggram.deviceid();
    // 如果能够获取stream
    if let Some(ostm) = ClientPoolManager::get_client(clt, dev).await {
        // 并且pool里的和传递过来的是同一个stream
        if Arc::ptr_eq(&ostm, &stm) {
            // 更新client的超时信息
            ClientPoolManager::update_client(clt, dev).await;
            // 回复已发送信息
            send_message_receipt(reqmsggram, stm).await;
            // 将消息发送到事件队列中
            send_message_to_queue(reqmsgbuff, reqmsggram).await;
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
async fn send_message_notlogin(
    reqmsggram: &Arc<MessageDataGram>, 
    stm: Arc<Mutex<SendStream>>
) {
    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let cdg = CommandDataGram::create_command_gram_from_message_gram(
        vecu8.as_mut(),
        reqmsggram.as_ref()
    );
    cdg.set_returncode(ReturnCode::RETURN_NOT_LOGIN);

    let mut stream = stm.lock().await;
    stream.write_all(vecu8.as_mut()).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}
//
async fn send_message_twosession_nosame(
    reqmsggram: &Arc<MessageDataGram>,
    stm: Arc<Mutex<SendStream>>
) {
    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let cdg = CommandDataGram::create_command_gram_from_message_gram(
        vecu8.as_mut(),
        reqmsggram.as_ref()
    );
    cdg.set_returncode(ReturnCode::RETURN_TWO_SESSION_NO_SAME);

    let mut stream = stm.lock().await;
    stream.write_all(vecu8.as_mut()).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}
//
async fn send_message_receipt(
    reqmsggram: &Arc<MessageDataGram>, 
    stm: Arc<Mutex<SendStream>>
) {
    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let cdg = CommandDataGram::create_command_gram_from_message_gram(
        vecu8.as_mut(),
        reqmsggram.as_ref()
    );
    cdg.set_returncode(ReturnCode::RETURN_OK);

    let mut stream = stm.lock().await;
    stream.write_all(vecu8.as_mut()).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}
//
async fn send_message_to_queue(
    reqmsgbuff: &Arc<Bytes>,
    reqmsggram: &Arc<MessageDataGram>
) {
    let sender = MessageEventQueue::get_sender();
    let msgevent = sender.lock().await;
    match reqmsggram.receivertype() {
        ClientType::CLIENT_PEOPLE =>
            msgevent
                .send(MessageEvent::MessageReceive {
                    reqmsgbuff: reqmsgbuff.clone(),
                    reqmsggram: reqmsggram.clone(),
                }).await
                .expect("send event error!"),
        ClientType::CLIENT_GROUP =>
            msgevent
                .send(MessageEvent::GroupReceive {
                    reqmsgbuff: reqmsgbuff.clone(),
                    reqmsggram: reqmsggram.clone(),
                }).await
                .expect("send event error!"),
        ClientType::CLIENT_DEVICE =>
            msgevent
                .send(MessageEvent::DeviceReceive {
                    reqmsgbuff: reqmsgbuff.clone(),
                    reqmsggram: reqmsggram.clone(),
                }).await
                .expect("send event error!"),
        ClientType::CLIENT_ROBOT =>
            msgevent
                .send(MessageEvent::RobotReceive {
                    reqmsgbuff: reqmsgbuff.clone(),
                    reqmsggram: reqmsggram.clone(),
                }).await
                .expect("send event error!"),
        ClientType::CLIENT_SERVICE =>
            msgevent
                .send(MessageEvent::ServiceReceive {
                    reqmsgbuff: reqmsgbuff.clone(),
                    reqmsggram: reqmsggram.clone(),
                }).await
                .expect("send event error!"),
        _ => {}
    }
}
