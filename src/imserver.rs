// 版权归亚马逊公司及其关联公司所有。保留所有权利。
// SPDX-License-Identifier: Apache-2.0

use btcmbase::datagram::{CommandDataGram, DataGramError, MessageDataGram, BitCommand, BitcommDataGram};
use bytes::Bytes;
#[allow(unused_imports)]
use s2n_quic::{Server, stream::{BidirectionalStream, SendStream}};
use tokio::io::AsyncWriteExt;
// use core::slice::SlicePattern;
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
    #[allow(unused_variables)]
    let cpmm = ClientPoolManager::new();
    // 创建多任务,可共享,可修改的ClientPoolManager
    #[allow(unused_variables)]
    let cpm = Arc::new(tokio::sync::Mutex::new(ClientPoolManager::new()));
    // 等待客户端连接
    while let Some(mut connection) = server.accept().await {
        #[allow(unused_variables)]
        // 为连接生成新任务,在新任务中必须clone
        let cpm = cpm.clone();
        // 生成异步新的任务
        tokio::spawn(async move {
            eprintln!("Connection accepted from {:?}", connection.remote_addr());
            // 从连接中获取双向流
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                let (mut receive_stream, mut send_stream) = stream.split();
                // 在这里获取stream id,后期不用lock获取
                let stmid = send_stream.id();
                // 组装共享器
                let stm = Arc::new(send_stream);
                // clone ClientPoolManager
                let cpm = cpm.clone();
                // 为流生成新异步新任务
                tokio::spawn(async move {
                    while let Ok(Some(data)) = receive_stream.receive().await {
                        // 将获取到的数据解包,生成信令报文或是消息报文,
                        let bitcomm_gram = handle_receive(data);
                        process_receive(stmid,bitcomm_gram,cpm,stm);
                        // 生成Response Gram
                        // handle_receive(stmid,cpm.clone(),stm.clone());
                        // receive_stream.connection().close(u32::from(0));
                        break;
                    }
                    
                });
            }
        });
    }
    Ok(())
}

fn process_receive<'a>(stmid   :u64,
    data    :Option<BitcommDataGram>,
    cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
    stm     :Arc<SendStream>) {
    stm.write_all(data.unwrap().req_cmdu8().unwrap());
}

fn handle_receive<'a>(data    :Bytes) ->Option<BitcommDataGram<'a>> {
    let byte_array = data.as_ref();
    // let stream = stream.lock().await;
    // 如果是命令报文
    if CommandDataGram::is_command_from_bytes(byte_array) {
        handle_command_data(&data)
    // 如果是消息报文
    } else if MessageDataGram::is_message_from_bytes(byte_array) {
        handle_message_data(&data)
    } else { // 如果两种报文都不是
        Option::None
    }
}

fn handle_command_data<'a>(data:&Bytes) -> Option<BitcommDataGram<'a>> {
    // 生成command消息体
    let command = CommandDataGram::get_command_data_gram_by_u8(data);
    // 处理login命令
    if command.command().contains(BitCommand::LOGIN_COMMAND) {
        login::handle_command_login(data,command)    
    } else if command.command().contains(BitCommand::LOGOUT_COMMAND) {
        logout::handle_command_logout(data,command)  
    } else {
        Option::None
    }
}

#[allow(unused_variables)]
fn handle_message_data<'a>(data    :&bytes::Bytes) -> Option<BitcommDataGram<'a>> {
    let message = MessageDataGram::get_message_data_gram_by_u8(data);
        // 新建一个CommandDataGram
    let mut vecu8 = MessageDataGram::create_gram_buf(0);
    let resmessage = MessageDataGram::create_message_data_gram_from_mdg_u8(vecu8.as_mut_slice(), message);
    // 新建一个CommandDataGram
    
    eprintln!("Stream opened gram    from {:?}", resmessage);
    Option::Some(BitcommDataGram::new_message_data_gram(Option::Some(message), 
                                                      Option::Some(data.as_ref()),
                                                        Option::Some(resmessage),
                                                      Option::Some(vecu8.as_slice()),))
}












fn handle_receive1(stmid   :u64,
                        data    :Bytes,
                        cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                        stm     :Arc<tokio::sync::Mutex<SendStream>>) {
    // let mut stream = stm.lock().await;
    // let mut stream = wrp.0;
    // 等待流中的数据
    // while let Ok(Some(data)) = stream.receive().await {
        eprintln!("Stream opened data    from {:?}", data);
        match process_data(&data,stmid, cpm.clone(),stm.clone()) {
            Ok(Some(datagram)) => {
                eprintln!("DataGram    from {:?}", datagram);
                // stream.write_all(datagram.as_ref()).await.expect("stream should be open");
                // stream.send(datagram).await.expect("stream should be open");
                // stream.flush().await.expect("stream should be open");
            }
            Ok(None) => {
                // stream.send(data).await.expect("stream should be open");
            }
            Err(err) => {
                eprintln!("DataGramError = {:?}", err);
            }
        }
    // }
}


fn process_data(data      :&bytes::Bytes,
                        stmid   :u64,
                        cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                        stream  :Arc<tokio::sync::Mutex<SendStream>>) 
                        -> Result<Option<bytes::Bytes>,DataGramError> {

    let byte_array = data.as_ref();
    // let stream = stream.lock().await;
    // 如果是命令报文
    if CommandDataGram::is_command_from_bytes(byte_array) {
        let rs = process_command_data(data,stmid,cpm,stream).await;
        Result::Ok(rs)
    // 如果是消息报文
    } else if MessageDataGram::is_message_from_bytes(byte_array) {
        let rs = process_message_data(data,stmid,cpm,stream).await;
        Result::Ok(rs)
    } else { // 如果两种报文都不是
        Result::Ok(Option::None)
    }
}

fn process_command_data(data  :&bytes::Bytes,
                            stmid   :u64,
                            cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                            stream  :Arc<tokio::sync::Mutex<SendStream>>) 
                        -> Option<bytes::Bytes> {
    // 生成command消息体
    let command = CommandDataGram::get_command_data_gram_by_u8(data);
    // 处理login命令
    if command.command().contains(BitCommand::LOGIN_COMMAND) {
        login::process_command_login(command, stmid, cpm, stream)    
    } else if command.command().contains(BitCommand::LOGOUT_COMMAND) {
        logout::process_command_logout(command, stmid, cpm, stream)  
    } else {
        Option::None
    }
}



#[allow(unused_variables)]
fn process_message_data(data    :&bytes::Bytes,
                        stmid   :u64,
                        cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                        stream  :Arc<tokio::sync::Mutex<SendStream>>) 
                        -> Option<bytes::Bytes> {
    let message = MessageDataGram::get_message_data_gram_by_u8(data);
        // 新建一个CommandDataGram
    let mut vecu8 = MessageDataGram::create_gram_buf(0);
    let resmessage = MessageDataGram::create_message_data_gram_from_mdg_u8(vecu8.as_mut_slice(), message);
    // 新建一个CommandDataGram
    
    eprintln!("Stream opened gram    from {:?}", resmessage);
    Option::Some(Bytes::from(vecu8))
}