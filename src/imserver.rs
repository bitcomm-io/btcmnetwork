// 版权归bitcomm.io公司及其关联公司所有。保留所有权利。
// SPDX-License-Identifier: Apache-2.0

use btcmbase::datagram::{ CommandDataGram, DataGramError, InnerDataGram, MessageDataGram };
use bytes::Bytes;
#[allow(unused_imports)]
use s2n_quic::{ stream::{ BidirectionalStream, SendStream }, Server };
// use tokio::sync::{ mpsc::Sender, Mutex };

use std::{ error::Error, sync::Arc, time::Duration };
// use tokio::sync::Mutex;
use crate::{
    // connservice::ClientPoolManager,
    // eventqueue::MessageEvent,
    procommand,
    promessage,
    propingpong,
    slowloris,
};
use btcmtools::LOGGER;
use slog::info;

pub static BITCOMM_IMSERVER     :&str = "0.0.0.0";
pub static BITCOMM_IMSERVER_PORT:&str = "1130"; 

/// 
fn get_imserver_port() -> String {
    format!("{}:{}", BITCOMM_IMSERVER, BITCOMM_IMSERVER_PORT)
}

/// 注意：此证书仅供演示目的使用！
pub static CERT_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../certs/cert.pem"));
/// 注意：此密钥仅供演示目的使用！
pub static KEY_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../certs/key.pem"));

/// 获取服务器实例
fn get_server() -> Result<Server, Box<dyn Error>> {
    let connection_limits = s2n_quic::provider::limits::Limits
        ::new()
        .with_max_handshake_duration(Duration::from_secs(5))
        .expect("connection limits are valid");
    let endpoint_limits = s2n_quic::provider::endpoint_limits::Default
        ::builder()
        .with_inflight_handshake_limit(100)?
        .build()?;
    let server_address = get_imserver_port();
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
        .with_io(server_address.as_str())?
        .start()?;
    info!(LOGGER, "quic listening on {}", server.local_addr().unwrap());
    Ok(server)
}

/// 开启即时消息服务器
///
/// # Arguments
///
/// * `None`
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - 成功启动即时消息服务器或错误信息
///
/// # Examples
///
/// ```rust
/// let result = start_instant_message_server().await;
/// match result {
///     Ok(()) => println!("Instant message server started successfully"),
///     Err(err) => eprintln!("Error starting instant message server: {}", err),
/// }
/// ```
pub async fn start_instant_message_server() -> Result<(), Box<dyn Error>> {
    // 获取服务器实例
    let mut server = get_server()?;
    // 接受客户端连接并处理
    while let Some(mut connection) = server.accept().await {
        // 设置连接不超时
        connection.keep_alive(true)?;

        // 克隆共享资源
        // let cpm1 = cpm0.clone();
        // let meqsend1 = meqsend0.clone();

        // 异步处理连接
        tokio::spawn(async move {
            // 记录连接接受日志
            slog::info!(
                btcmtools::LOGGER,
                "Connection accepted from {:?}",
                connection.remote_addr()
            );

            // 接受双向流
            while let Ok(Some(stream)) = connection.accept_bidirectional_stream().await {
                // 分割流为接收流和发送流
                let (mut receive_stream, send_stream) = stream.split();
                // 创建发送流的互斥锁
                let stm0 = Arc::new(tokio::sync::Mutex::new(send_stream));
                // 异步处理数据流
                tokio::spawn(async move {
                    // 接收数据并处理
                    while let Ok(Some(reqbuff)) = receive_stream.receive().await {
                        let rcreqbuff = Arc::new(reqbuff);
                        // 准备数据缓冲区
                        if let Some(inner_data_gram) = prepare_data_buffer(rcreqbuff.clone()) {
                            let rcdatagram = Arc::new(inner_data_gram);
                            // 处理数据
                            handle_data(rcdatagram.clone());
                            // 异步处理数据
                            if let Err(err) = process_data(rcdatagram.clone(), stm0.clone()).await {
                                slog::error!(btcmtools::LOGGER, "process data error: {}", err);
                            }
                        } else {
                            // 获取发送流的互斥锁并发送数据
                            let mut send_stream = stm0.lock().await;
                            // 记录客户端主机信息和接收到的数据
                            slog::info!(
                                btcmtools::LOGGER,
                                "client host from {:?}",
                                send_stream.connection().remote_addr()
                            );
                            slog::info!(
                                btcmtools::LOGGER,
                                "receive data is  {:?}",
                                rcreqbuff.as_ref()
                            );
                            // 发送数据并处理错误
                            if
                                let Err(err) = send_stream.send(
                                    Arc::try_unwrap(rcreqbuff).unwrap()
                                ).await
                            {
                                slog::error!(btcmtools::LOGGER, "send error: {}", err);
                            }
                        }
                    }
                    // 在此需要清理IP->ClientID+DeviceID信息
                });
            }
        });
    }
    Ok(())
}

fn prepare_data_buffer(reqbuff: Arc<Bytes>) -> Option<InnerDataGram> {
    let bts = reqbuff.as_ref();

    if CommandDataGram::is_bitcomm_flag(bts) {
        let bitcomm_flag = CommandDataGram::get_bitcomm_flag_by_u8(bts);
        return Some(InnerDataGram::Pingpong(Arc::new(*bitcomm_flag)));
    }

    if CommandDataGram::is_command_from_bytes(bts) {
        let reqcmdgram = CommandDataGram::get_command_data_gram_by_u8(bts);
        return Some(InnerDataGram::Command {
            reqcmdbuff: reqbuff.clone(),
            reqcmdgram: Arc::new(*reqcmdgram),
        });
    }

    if MessageDataGram::is_message_from_bytes(bts) {
        let reqmsggram = MessageDataGram::get_message_data_gram_by_u8(bts);
        return Some(InnerDataGram::Message {
            reqmsgbuff: reqbuff.clone(),
            reqmsggram: Arc::new(*reqmsggram),
        });
    }
    None
}

#[allow(unused_variables)]
fn handle_data(datagram: Arc<InnerDataGram>) {
    match datagram.as_ref() {
        InnerDataGram::Command { reqcmdbuff, reqcmdgram } => {
            procommand::handle_command_data(reqcmdbuff, reqcmdgram);
        }
        InnerDataGram::Message { reqmsgbuff, reqmsggram } => {
            promessage::handle_message_data(reqmsgbuff, reqmsggram);
        }
        InnerDataGram::Pingpong(pingpong) => {}
    }
}
/// 处理数据并返回结果
///
/// # Arguments
///
/// * `data` - 数据包
/// * `stm` - 发送流的互斥锁
///
/// # Returns
///
/// * `Result<Arc<InnerDataGram>, DataGramError>` - 成功处理数据并返回结果或错误信息
///
/// # Examples
///
/// ```rust
/// let data = Arc::new(InnerDataGram::Command {
///     reqcmdbuff: Arc::new(Bytes::from_static(&[0x01, 0x02, 0x03])),
///     reqcmdgram: Arc::new(CommandDataGram::default()),
/// });
/// let stm = Arc::new(tokio::sync::Mutex::new(SendStream::default()));
/// let result = process_data(data, stm).await;
/// match result {
///     Ok(data) => println!("Data processed successfully: {:?}", data),
///     Err(err) => eprintln!("Error processing data: {}", err),
/// }
/// ```
#[allow(unused_variables)]
async fn process_data<'a>(
    data: Arc<InnerDataGram>,
    stm: Arc<tokio::sync::Mutex<SendStream>>
) -> Result<Arc<InnerDataGram>, DataGramError> {
    match data.as_ref() {
        InnerDataGram::Command { reqcmdbuff, reqcmdgram } =>
            procommand::process_command_data(reqcmdbuff, reqcmdgram, stm).await,

        InnerDataGram::Message { reqmsgbuff, reqmsggram } =>
            promessage::process_message_data(reqmsgbuff, reqmsggram, stm).await,

        InnerDataGram::Pingpong(pingpong) =>
            propingpong::process_bitcomm_flag_data(pingpong, stm).await,
    }
    Result::Ok(data)
}
