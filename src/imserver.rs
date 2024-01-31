// 版权归亚马逊公司及其关联公司所有。保留所有权利。
// SPDX-License-Identifier: Apache-2.0

use btcmbase::datagram::{CommandDataGram, DataGramError, MessageDataGram, BitCommand, InnerDataGram};
use bytes::Bytes;
#[allow(unused_imports)]
use s2n_quic::{Server, stream::{BidirectionalStream, SendStream}};
// use tokio::io::AsyncWriteExt;

use std::{error::Error, time::Duration, sync::Arc};
// use tokio::sync::Mutex;
use crate::{slowloris, connservice::ClientPoolManager,login, logout};



// use std::future::Future;
// use crate::slowloris::MyConnectionSupervisor;
/// 注意：此证书仅供演示目的使用！
pub static CERT_PEM: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../certs/cert.pem"
));
/// 注意：此密钥仅供演示目的使用！
pub static KEY_PEM: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../certs/key.pem"
));


pub async fn start_instant_message_server() -> Result<(), Box<dyn Error>> {
    // 限制任何握手尝试的持续时间为5秒
    // 默认情况下，握手的限制时间为10秒。
    let connection_limits = s2n_quic::provider::limits::Limits::new()
        .with_max_handshake_duration(Duration::from_secs(5))
        .expect("connection limits are valid");

    // 限制正在进行的握手次数为100。
    let endpoint_limits = s2n_quic::provider::endpoint_limits::Default::builder()
        .with_inflight_handshake_limit(100)?
        .build()?;

    // 构建`s2n_quic::Server`
    let mut server = Server::builder()
        // 提供上述定义的`connection_limits`
        .with_limits(connection_limits)?
        // 提供上述定义的`endpoint_limits`
        .with_endpoint_limits(endpoint_limits)?
        // 提供由`dos-mitigation/src/lib.rs`中定义的`slowloris::MyConnectionSupervisor`和默认事件跟踪订阅者组成的元组。
        // 此组合将允许利用`MyConnectionSupervisor`的slowloris缓解功能以及事件跟踪。
        .with_event((
            slowloris::MyConnectionSupervisor,
            s2n_quic::provider::event::tracing::Subscriber::default(),
        ))?
        .with_tls((CERT_PEM, KEY_PEM))?
        // .with_io("127.0.0.1:4433")?
        .with_io("0.0.0.0:4433")?
        .start()?;
    // #[allow(unused_variables)]
    // let cpmm = ClientPoolManager::new();
    // 创建多任务,可共享,可修改的ClientPoolManager
    // #[allow(unused_variables)]
    let cpm0 = Arc::new(tokio::sync::Mutex::new(ClientPoolManager::new()));
    // 等待客户端连接
    while let Some(mut connection) = server.accept().await {
        #[allow(unused_variables)]
        // 为连接生成新任务,在新任务中必须clone
        let cpm1 = cpm0.clone();
        // 生成异步新的任务
        tokio::spawn(async move {
            eprintln!("Connection accepted from {:?}", connection.remote_addr());
            // 从连接中获取双向流
            #[allow(unused_mut)]
            // #[allow(unused_variables)]
            while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                let (mut receive_stream, mut send_stream) = stream.split();
                // 在这里获取stream id,后期不用lock获取
                let stmid = send_stream.id();
                // 组装共享器
                let stm0 = Arc::new(tokio::sync::Mutex::new(send_stream));
                // clone ClientPoolManager
                let cpm2 = cpm1.clone();
                // 为流生成新异步新任务
                tokio::spawn(async move {
                    // 获取收到数据的缓冲区
                    while let Ok(Some(reqbuff)) = receive_stream.receive().await {
                        let rcreqbuff = Arc::new(reqbuff);
                        // 将req,res两个数据区进行结构整理,形成内部报文结构
                        if let Some(mut inner_data_gram) = prepare_data_buffer(rcreqbuff.clone()) {
                            let rcdatagram = Arc::new(inner_data_gram);
                            // 将获取到的数据解包,生成信令报文或是消息报文,同步方式的预处理
                            handle_receive(rcdatagram.clone());
                            // 异步方式的处理
                            process_data(stmid,rcdatagram.clone(),cpm2.clone(),stm0.clone()).await.expect("process data error");                       
                        } else {
                            let mut send_stream = stm0.lock().await;
                            eprintln!("client host from {:?}", send_stream.connection().remote_addr());
                            eprintln!("client host from {:?}", rcreqbuff.as_ref());
                            send_stream.send(Arc::try_unwrap(rcreqbuff).unwrap()).await.expect("");
                        }
                    }
                    
                });
            }
        });
    }
    Ok(())
}

fn prepare_data_buffer(reqbuff :Arc<Bytes>) -> Option<InnerDataGram> {
    // 如果是命令报文
    if CommandDataGram::is_command_from_bytes(reqbuff.as_ref()) {
        let bts = reqbuff.as_ref();
        let reqcmdgram = CommandDataGram::get_command_data_gram_by_u8(bts.as_ref());
        let reqdata = InnerDataGram::Command {reqcmdbuff:reqbuff.clone(),reqcmdgram:Arc::new(*reqcmdgram)};
        Some(reqdata)
    // 如果是消息报文
    } else if MessageDataGram::is_message_from_bytes(reqbuff.as_ref()) {
        let reqmsggram = MessageDataGram::get_message_data_gram_by_u8(reqbuff.as_ref());
        let reqdata = InnerDataGram::Message {reqmsgbuff:reqbuff.clone(),reqmsggram:Arc::new(*reqmsggram)};
        Some(reqdata)
    } else { // 如果两种报文都不是
        Option::None
    }
}

#[allow(unused_variables)]
fn handle_receive(datagram:Arc<InnerDataGram>) {

    match datagram.as_ref() {
        InnerDataGram::Command{reqcmdbuff,reqcmdgram} => {
            handle_command_data(reqcmdbuff,reqcmdgram);
        }
        InnerDataGram::Message { reqmsgbuff, reqmsggram } => {
            handle_message_data(reqmsgbuff, reqmsggram);
        }
    }
}

fn handle_command_data<'a>(reqcmdbuff:&Arc<Bytes>,reqcmdgram:&Arc<CommandDataGram>) {
    // 处理login命令
    if reqcmdgram.command().contains(BitCommand::LOGIN_COMMAND) {
        login::handle_command_login(reqcmdbuff,reqcmdgram)    
    } else if reqcmdgram.command().contains(BitCommand::LOGOUT_COMMAND) {
        logout::handle_command_logout(reqcmdbuff,reqcmdgram)  
    }
}

#[allow(unused_variables)]
fn handle_message_data(reqmsgbuff:&Arc<Bytes>,reqmsggram:&Arc<MessageDataGram>) {
    // let message = MessageDataGram::get_message_data_gram_by_u8(reqdata);
        // 新建一个CommandDataGram
    // let mut vecu8 = MessageDataGram::create_gram_buf(0);
    // let resmessage = MessageDataGram::create_message_data_gram_from_mdg_u8(vecu8.as_mut_slice(), message);
    // 新建一个CommandDataGram
    // let dd = data.as_ref();
    // eprintln!("Stream opened gram    from {:?}", resmessage);
    // None
    // Option::Some(BitcommDataGram::new_message_data_gram(Option::Some(message), 
    //                                                   Option::Some(data),
    //                                                     Option::Some(resmessage),
    //                                                   Option::Some(vecu8)))
}








#[allow(unused_variables)]
async fn process_data<'a>(stmid   :u64,
                          data    :Arc<InnerDataGram>,
                          cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                          stm     :Arc<tokio::sync::Mutex<SendStream>>) -> Result<Arc<InnerDataGram>,DataGramError>{
    // let stream = stream.lock().await;
    match data.as_ref() {
        InnerDataGram::Command { reqcmdbuff, reqcmdgram } => {
            process_command_data(stmid,reqcmdbuff, reqcmdgram,cpm,stm).await;
        }
        InnerDataGram::Message { reqmsgbuff, reqmsggram } => {
            process_message_data(stmid,reqmsgbuff, reqmsggram,cpm,stm);
        }
    }
    Result::Ok(data)
}


async fn process_command_data<'a>(stmid   :u64,
                                reqcmdbuff:&Arc<Bytes>,reqcmdgram:&Arc<CommandDataGram>,
                                cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                                stm     :Arc<tokio::sync::Mutex<SendStream>>) {
    // 处理login命令
    if reqcmdgram.command().contains(BitCommand::LOGIN_COMMAND) {
        login::process_command_login(stmid,reqcmdbuff,reqcmdgram,cpm,stm).await;
    } else if reqcmdgram.command().contains(BitCommand::LOGOUT_COMMAND) { // 处理登出logout
        logout::process_command_logout(stmid,reqcmdbuff,reqcmdgram,cpm,stm).await; 
    } 
}



#[allow(unused_variables)]
fn process_message_data<'a>(stmid   :u64,
                        reqmsgbuff:&Arc<Bytes>,reqmsggram:&Arc<MessageDataGram>,
                        cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                        stm     :Arc<tokio::sync::Mutex<SendStream>>) {

}