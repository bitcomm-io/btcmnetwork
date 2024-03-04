use std::sync::Arc;
use btcmbase::datagram::CommandDataGram;
use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::io::AsyncWriteExt;

use crate::connservice::ClientPoolManager;

/// 异步处理命令 pingpong
///
/// 此函数用于异步处理命令 pingpong。它接受命令缓冲区、命令数据报和发送流互斥锁作为参数，
/// 并将命令 pingpong 发送到客户端，并缓存客户端的信息。
///
/// # Arguments
///
/// * `reqcmdbuff` - 命令缓冲区的 Arc<Bytes> 引用
/// * `reqcmdgram` - 命令数据报的 Arc<CommandDataGram> 引用
/// * `stm` - 发送流的互斥锁的 Arc<tokio::sync::Mutex<SendStream>> 引用
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use btcmbase::datagram::CommandDataGram;
/// use bytes::Bytes;
/// use s2n_quic::stream::SendStream;
/// use tokio::sync::Mutex;
/// use crate::connservice::ClientPoolManager;
///
/// let reqcmdbuff = Arc::new(Bytes::new());
/// let reqcmdgram = Arc::new(CommandDataGram::default());
/// let stm = Arc::new(Mutex::new(SendStream::default()));
/// process_command_pingpong(&reqcmdbuff, &reqcmdgram, stm).await;
/// ```
#[allow(unused_variables)]
pub async fn process_command_pingpong(
    reqcmdbuff: &Arc<Bytes>,
    reqcmdgram: &Arc<CommandDataGram>,
    stm: Arc<tokio::sync::Mutex<SendStream>>
) {
    // 记录日志
    slog::info!(btcmtools::LOGGER, "processing command pingpong: {:?}", reqcmdgram);

    // 缓存客户端信息
    ClientPoolManager::put_client(reqcmdgram.sender().into(), reqcmdgram.deviceid(), stm.clone()).await;

    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let _cdg = CommandDataGram::create_command_gram_from_gram(vecu8.as_mut(), reqcmdgram.as_ref());

    let mut stream = stm.lock().await;
    let u8array = vecu8.as_mut();
    // 将命令 pingpong 发送到客户端
    stream.write_all(u8array).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}

// use std::sync::Arc;

// use btcmbase::datagram::CommandDataGram;
// use bytes::Bytes;
// use s2n_quic::stream::SendStream;
// use tokio::io::AsyncWriteExt;

// use crate::connservice::ClientPoolManager;

// #[allow(unused_variables)]
// pub async fn process_command_pingpong<'a>(
//     reqcmdbuff: &Arc<Bytes>,
//     reqcmdgram: &Arc<CommandDataGram>,
//     stm: Arc<tokio::sync::Mutex<SendStream>>
// ) {
//     // let command = data.req_cmdgram.unwrap();
//     // 此方法中需要对 token 进行验证
//     slog::info!(btcmtools::LOGGER, "client login server {:?}", reqcmdgram);
//     // let mut ccp = cpm.lock().await;
//     // 缓存client的信息
//     ClientPoolManager::put_client(reqcmdgram.sender().into(), reqcmdgram.deviceid(), stm.clone()).await;
//     // 必须clone
//     // ccp.put_stream(stmid, stm.clone());

//     let mut vecu8 = CommandDataGram::create_gram_buf(0);
//     let cdg = CommandDataGram::create_command_gram_from_gram(vecu8.as_mut(), reqcmdgram.as_ref());

//     let mut stream = stm.lock().await;
//     // let mut bts = rescmdbuff.as_ref();
//     let u8array = vecu8.as_mut();
//     stream.write_all(u8array).await.expect("stream should be open");
//     // stream.write_all(u8array).await.expect("stream should be open");
//     stream.flush().await.expect("stream should be open");
// }
