// 版权归亚马逊公司及其关联公司所有。保留所有权利。
// SPDX-License-Identifier: Apache-2.0

use btcmbase::datagram::{CommandDataGram, DataGramError, MessageDataGram};
use bytes::Bytes;
use s2n_quic::Server;
use tokio::io::AsyncWriteExt;
use std::{error::Error, time::Duration};
use crate::slowloris;
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

    while let Some(mut connection) = server.accept().await {
        // 为连接生成新任务
        tokio::spawn(async move {
            eprintln!("Connection accepted from {:?}", connection.remote_addr());
            
            while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                // 为流生成新任务
                tokio::spawn(async move {
                    eprintln!("Stream opened from {:?}", stream.connection().remote_addr());
                    
                    while let Ok(Some(data)) = stream.receive().await {
                        eprintln!("Stream opened data    from {:?}", data);
                        match process_data(&data) {
                            Ok(Some(datagram)) => {
                                eprintln!("DataGram    from {:?}", datagram);
                                stream.write_all(datagram.as_ref()).await.expect("stream should be open");
                                // stream.send(datagram).await.expect("stream should be open");
                                stream.flush().await.expect("stream should be open");
                            }
                            Ok(None) => {
                                stream.send(data).await.expect("stream should be open");
                            }
                            Err(err) => {
                                eprintln!("DataGramError = {:?}", err);
                            }
                        }
                    }
                });
            }
        });
    }

    Ok(())
}

fn process_data(data:&bytes::Bytes) -> Result<Option<Bytes>,DataGramError> {
    let byte_array = data.as_ref();
    // 如果是命令报文
    if CommandDataGram::is_command_from_bytes(byte_array) {
        process_command_data(data)
    // 如果是消息报文
    } else if MessageDataGram::is_message_from_bytes(byte_array) {
        process_message_data(data)
    } else { // 如果两种报文都不是
        Result::Ok(Option::None)
    }
    // let data_gram_head: & CommandDataGram = CommandDataGram::get_command_data_gram_by_bytes(data);
    
    // if data_gram_head.bitcomm() == CommandDataGram::BITCOMM_COMMAND {
    //     let mut vec_u8: Vec<u8> = vec![0x00; CommandDataGram::get_size()];
    //     Result::Ok(Option::Some(Bytes::from(vec_u8)))
    // } else {
    //     Result::Err(DataGramError{errcode:0xFF,details:"no data gram format!".to_string()})
    // } 
}
fn process_command_data(data:&bytes::Bytes) -> Result<Option<Bytes>,DataGramError> {
    let command = CommandDataGram::get_command_data_gram_by_u8(data);
    // 新建一个CommandDataGram
    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let rescommand = CommandDataGram::create_command_gram_from_message_gram(vecu8.as_mut_slice(), command);
    eprintln!("Stream opened gram    from {:?}", rescommand);
    Result::Ok(Option::Some(Bytes::from(vecu8)))
}
fn process_message_data(data:&bytes::Bytes) -> Result<Option<Bytes>,DataGramError> {
    let message = MessageDataGram::get_message_data_gram_by_u8(data);
        // 新建一个CommandDataGram
    let mut vecu8 = MessageDataGram::create_gram_buf(0);
    let resmessage = MessageDataGram::create_message_data_gram_from_mdg_u8(vecu8.as_mut_slice(), message);
    // 新建一个CommandDataGram
    eprintln!("Stream opened gram    from {:?}", resmessage);
    Result::Ok(Option::Some(Bytes::from(vecu8)))
}