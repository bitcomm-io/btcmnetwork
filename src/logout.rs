#[allow(unused_imports)]
use std::sync::Arc;

use btcmbase::datagram::CommandDataGram;
use bytes::Bytes;
use s2n_quic::stream::SendStream;
#[allow(unused_imports)]
use tokio::io::AsyncWriteExt;
use tracing::info;
// use tracing::error;
use crate::connservice::ClientPoolManager;

/// 处理命令登出请求。
///
/// # 参数
/// - `reqcmdbuff`: 请求命令的字节数据。
/// - `reqcmdgram`: 请求命令的数据报。
#[allow(unused_variables)]
pub fn handle_command_logout(reqcmdbuff: &Arc<Bytes>, reqcmdgram: &Arc<CommandDataGram>) {
    // 待处理
}

/// 处理命令登出请求的异步版本。
///
/// # 参数
/// - `reqcmdbuff`: 请求命令的字节数据。
/// - `reqcmdgram`: 请求命令的数据报。
/// - `stm`: 发送流的互斥锁。
#[allow(unused_variables)]
pub async fn process_command_logout<'a>(
    reqcmdbuff: &Arc<Bytes>,
    reqcmdgram: &Arc<CommandDataGram>,
    stm: Arc<tokio::sync::Mutex<SendStream>>
) {
    // 记录日志
    info!("client logout server {:?}", reqcmdgram);

    // 创建命令数据报
    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let cdg = CommandDataGram::create_command_gram_from_gram(vecu8.as_mut(), reqcmdgram.as_ref());

    // 发送数据到客户端
    let mut stream = stm.lock().await;
    let u8array = vecu8.as_mut();
    stream.write_all(u8array).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
    // 清除
    ClientPoolManager::remove_addres(&stream.connection().remote_addr().unwrap().to_string()).await;
    // 关闭流和连接
    stream.close().await.expect("close stream error!");
    stream.connection().close(LOGOUT_CODE.into());

    // 移除客户端
    // let os = ClientPoolManager::remove_client(reqcmdgram.sender().into(),reqcmdgram.deviceid()).await;
    if
        let Some(ostm) = ClientPoolManager::remove_client(
            reqcmdgram.sender().into(),
            reqcmdgram.deviceid()
        ).await
    {
        // 关闭旧流和连接
        if !Arc::ptr_eq(&stm, &ostm) {
            let mut ostm = ostm.lock().await;
            // 清除
            ClientPoolManager::remove_addres(&ostm.connection().remote_addr().unwrap().to_string()).await;
            ostm.close().await.expect("close ostream error!");
            ostm.connection().close(LOGOUT_CODE.into());
        }
    }
}

// 登出代码
const LOGOUT_CODE: u32 = 0x0009;

// #[allow(unused_imports)]
// use std::{sync::Arc, rc::Rc};

// use btcmbase::datagram::CommandDataGram;
// use bytes::Bytes;
// use s2n_quic::stream::SendStream;
// #[allow(unused_imports)]
// use tokio::io::AsyncWriteExt;

// use crate::connservice::ClientPoolManager;
// #[allow(unused_variables)]
// pub fn handle_command_logout(reqcmdbuff:&Arc<Bytes>,reqcmdgram:&Arc<CommandDataGram>) {
// // let mut ve = res.res_cmdu8.as_ref().unwrap();
// // let rescommand = CommandDataGram::create_command_gram_from_message_gram(ve.as_mut_slice(), command);

//   // Option::Some(BitcommDataGram::new_command_data_gram(Option::Some(reqcmd),
//   //                                                   Option::Some(reqdata),
//   //                                                     Option::Some(rescmd),
//   //                                                   Option::Some(resdata),))
// }

// #[allow(unused_variables)]
// pub async fn process_command_logout<'a>(
//     reqcmdbuff:&Arc<Bytes>,reqcmdgram:&Arc<CommandDataGram>,
//     stm     :Arc<tokio::sync::Mutex<SendStream>>
// ) {
//     // let command = data.req_cmdgram.unwrap();
//     // 此方法中需要对 token 进行验证
//     slog::info!(btcmtools::LOGGER,"client logout server {:?}", reqcmdgram);
//     // tokio::runtime::Runtime::new().unwrap().block_on(async {
//         // let mut ccp = cpm.lock().await;

//         let mut vecu8 = CommandDataGram::create_gram_buf(0);
//         let cdg = CommandDataGram::create_command_gram_from_gram(vecu8.as_mut(), reqcmdgram.as_ref());

//         let mut stream = stm.lock().await;
//         let u8array = vecu8.as_mut();
//         stream.write_all(u8array).await.expect("stream should be open");
//         stream.flush().await.expect("stream should be open");

//         stream.close().await.expect("close stream error!");
//         stream.connection().close(LOGOUT_CODE.into());

//         let os = ClientPoolManager::remove_client(reqcmdgram.sender().into(),reqcmdgram.deviceid()).await;
//         if let Some(ostm) = os {
//           //
//           if !Arc::ptr_eq(&stm, &ostm) {
//             let mut ostm = ostm.lock().await;
//             ostm.close().await.expect("close ostream error!");
//             ostm.connection().close(LOGOUT_CODE.into());
//           }
//         }
//     // })
// }
// const LOGOUT_CODE:u32 = 0x0009;
