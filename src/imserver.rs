// 版权归bitcomm.io公司及其关联公司所有。保留所有权利。
// SPDX-License-Identifier: Apache-2.0

use btcmbase::datagram::{CommandDataGram, DataGramError, InnerDataGram, MessageDataGram};
use bytes::Bytes;
#[allow(unused_imports)]
use s2n_quic::{
    stream::{BidirectionalStream, SendStream},
    Server,
};
use tokio::sync::{mpsc::Sender, Mutex};

use std::{error::Error, sync::Arc, time::Duration};
// use tokio::sync::Mutex;
use crate::{
    connservice::ClientPoolManager, eventqueue::MessageEvent, procommand, promessage, slowloris,
};

// use std::future::Future;
// use crate::slowloris::MyConnectionSupervisor;
/// 注意：此证书仅供演示目的使用！
pub static CERT_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../certs/cert.pem"));
/// 注意：此密钥仅供演示目的使用！
pub static KEY_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../certs/key.pem"));
//
fn get_server() -> Result<Server, Box<dyn Error>> {
    let connection_limits = s2n_quic::provider::limits::Limits::new()
        .with_max_handshake_duration(Duration::from_secs(5))
        .expect("connection limits are valid");
    let endpoint_limits = s2n_quic::provider::endpoint_limits::Default::builder()
        .with_inflight_handshake_limit(100)?
        .build()?;
    let server = Server::builder()
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
        .with_io("0.0.0.0:9563")?
        .start()?;
    println!("quic listening on {}", server.local_addr().unwrap());
    Ok(server)
}

pub async fn start_instant_message_server(
    cpm0: Arc<tokio::sync::Mutex<ClientPoolManager>>,
    meqsend0: Arc<Mutex<Sender<MessageEvent>>>,
) -> Result<(), Box<dyn Error>> {
    // 获取服务器
    let mut server = get_server()?;
    // 等待客户端连接
    while let Some(mut connection) = server.accept().await {
        // 设置不超时
        connection.keep_alive(true)?;
        #[allow(unused_variables)]
        // 为连接生成新任务,在新任务中必须clone
        let cpm1 = cpm0.clone();
        let meqsend1 = meqsend0.clone();
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
                let meqsend2 = meqsend1.clone();
                // 为流生成新异步新任务
                tokio::spawn(async move {
                    // 获取收到数据的缓冲区
                    while let Ok(Some(reqbuff)) = receive_stream.receive().await {
                        let rcreqbuff = Arc::new(reqbuff);
                        // 将req,res两个数据区进行结构整理,形成内部报文结构
                        if let Some(mut inner_data_gram) = prepare_data_buffer(rcreqbuff.clone()) {
                            let rcdatagram = Arc::new(inner_data_gram);
                            // 将获取到的数据解包,生成信令报文或是消息报文,同步方式的预处理
                            handle_data(rcdatagram.clone());
                            // 异步方式的处理
                            process_data(
                                stmid,
                                rcdatagram.clone(),
                                cpm2.clone(),
                                stm0.clone(),
                                meqsend2.clone(),
                            )
                            .await
                            .expect("process data error");
                        } else {
                            let mut send_stream = stm0.lock().await;
                            eprint!(
                                "client host from {:?}",
                                send_stream.connection().remote_addr()
                            );
                            eprintln!("  data is  {:?}", rcreqbuff.as_ref());
                            send_stream
                                .send(Arc::try_unwrap(rcreqbuff).unwrap())
                                .await
                                .expect("");
                        }
                    }
                });
            }
        });
    }
    Ok(())
}

//
fn prepare_data_buffer(reqbuff: Arc<Bytes>) -> Option<InnerDataGram> {
    // 如果是命令报文
    if CommandDataGram::is_command_from_bytes(reqbuff.as_ref()) {
        let bts = reqbuff.as_ref();
        let reqcmdgram = CommandDataGram::get_command_data_gram_by_u8(bts.as_ref());
        let reqdata = InnerDataGram::Command {
            reqcmdbuff: reqbuff.clone(),
            reqcmdgram: Arc::new(*reqcmdgram),
        };
        Some(reqdata)
    // 如果是消息报文
    } else if MessageDataGram::is_message_from_bytes(reqbuff.as_ref()) {
        let reqmsggram = MessageDataGram::get_message_data_gram_by_u8(reqbuff.as_ref());
        let reqdata = InnerDataGram::Message {
            reqmsgbuff: reqbuff.clone(),
            reqmsggram: Arc::new(*reqmsggram),
        };
        Some(reqdata)
    } else {
        // 如果两种报文都不是
        Option::None
    }
}

#[allow(unused_variables)]
fn handle_data(datagram: Arc<InnerDataGram>) {
    match datagram.as_ref() {
        InnerDataGram::Command {
            reqcmdbuff,
            reqcmdgram,
        } => {
            procommand::handle_command_data(reqcmdbuff, reqcmdgram);
        }
        InnerDataGram::Message {
            reqmsgbuff,
            reqmsggram,
        } => {
            promessage::handle_message_data(reqmsgbuff, reqmsggram);
        }
    }
}
//
#[allow(unused_variables)]
async fn process_data<'a>(
    stmid: u64,
    data: Arc<InnerDataGram>,
    cpm: Arc<tokio::sync::Mutex<ClientPoolManager>>,
    stm: Arc<tokio::sync::Mutex<SendStream>>,
    meqsend: Arc<Mutex<Sender<MessageEvent>>>,
) -> Result<Arc<InnerDataGram>, DataGramError> {
    // let stream = stream.lock().await;
    match data.as_ref() {
        InnerDataGram::Command {
            reqcmdbuff,
            reqcmdgram,
        } => {
            procommand::process_command_data(stmid, reqcmdbuff, reqcmdgram, cpm, stm).await;
        }
        InnerDataGram::Message {
            reqmsgbuff,
            reqmsggram,
        } => {
            promessage::process_message_data(stmid, reqmsgbuff, reqmsggram, cpm, stm, meqsend)
                .await;
        }
    }
    Result::Ok(data)
}
