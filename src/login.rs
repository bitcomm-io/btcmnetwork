#[allow(unused_imports)]
use std::{ sync::Arc, rc::Rc };

use btcmbase::datagram::CommandDataGram;
use bytes::Bytes;
#[allow(unused_imports)]
use s2n_quic::stream::SendStream;
use tokio::io::AsyncWriteExt;
// use tokio::io::AsyncWriteExt;

use crate::connservice::ClientPoolManager;

/// 处理命令登录请求。
/// 
/// # 参数
/// - `reqcmdbuff`: 请求命令的字节数据。
/// - `reqcmdgram`: 请求命令的数据报。
#[allow(unused_variables)]
pub fn handle_command_login(reqcmdbuff: &Arc<Bytes>, reqcmdgram: &Arc<CommandDataGram>) {
}

/// 处理命令登录请求的异步版本。
/// 
/// # 参数
/// - `reqcmdbuff`: 请求命令的字节数据。
/// - `reqcmdgram`: 请求命令的数据报。
/// - `stm`: 发送流的互斥锁。
#[allow(unused_variables)]
pub async fn process_command_login<'a>(
    reqcmdbuff: &Arc<Bytes>,
    reqcmdgram: &Arc<CommandDataGram>,
    stm: Arc<tokio::sync::Mutex<SendStream>>
) {
    // 记录日志
    slog::info!(btcmtools::LOGGER, "client login server {:?}", reqcmdgram);
    
    let mut stream = stm.lock().await;
    
    // 向客户端发送数据
    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let cdg = CommandDataGram::create_command_gram_from_gram(vecu8.as_mut(), reqcmdgram.as_ref());
    let u8array = vecu8.as_mut();
    if let Err(err) = stream.write_all(u8array).await {
        slog::error!(btcmtools::LOGGER, "failed to write data to stream: {}", err);
        // return;
    }
    if let Err(err) = stream.flush().await {
        slog::error!(btcmtools::LOGGER, "failed to flush stream: {}", err);
        // return;
    }

    // 获取远程地址并添加到地址池
    if let Ok(address) = stream.connection().remote_addr() {
        let address_string = address.to_string();
        ClientPoolManager::put_addres(address_string.clone(), reqcmdgram.sender().into(), reqcmdgram.deviceid()).await;
    }
    // 缓存 client 信息,如果返回旧的ostem,则需要关闭
    if let Some(ostem) = ClientPoolManager::put_client(reqcmdgram.sender().into(), reqcmdgram.deviceid(), stm.clone()).await {
        let mut ostem = ostem.lock().await;
        ostem.close().await.unwrap();
        ostem.connection().close(OT_LOGIN_CODE.into());
        // ostem.connection().
    }
}
const OT_LOGIN_CODE: u32 = 0x000B;
// pub async fn process_command_login<'a>(
//     reqcmdbuff: &Arc<Bytes>,
//     reqcmdgram: &Arc<CommandDataGram>,
//     stm: Arc<tokio::sync::Mutex<SendStream>>
// ) {
//     // 记录日志
//     slog::info!(btcmtools::LOGGER, "client login server {:?}", reqcmdgram);
    
//     let mut stream = stm.lock().await;
    
//     // 向客户端发送数据
//     let mut vecu8 = CommandDataGram::create_gram_buf(0);
//     let cdg = CommandDataGram::create_command_gram_from_gram(vecu8.as_mut(), reqcmdgram.as_ref());
//     let u8array = vecu8.as_mut();
//     stream.write_all(u8array).await.expect("stream should be open");
//     stream.flush().await.expect("stream should be open");

//     // 获取远程地址并添加到地址池
//     let address = stream.connection().remote_addr().unwrap().to_string();
//     ClientPoolManager::put_addres(address, reqcmdgram.sender().into(), reqcmdgram.deviceid()).await.unwrap();
//     // 缓存 client 信息
//     ClientPoolManager::put_client(reqcmdgram.sender().into(), reqcmdgram.deviceid(), stm.clone()).await;
    
// }
