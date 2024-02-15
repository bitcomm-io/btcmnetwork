use std::sync::Arc;

use btcmbase::datagram::CommandDataGram;
use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::io::AsyncWriteExt;

use crate::connservice::ClientPoolManager;

#[allow(unused_variables)]
pub async fn process_command_pingpong<'a>(
    stmid: u64,
    reqcmdbuff: &Arc<Bytes>,
    reqcmdgram: &Arc<CommandDataGram>,
    cpm: Arc<tokio::sync::Mutex<ClientPoolManager>>,
    stm: Arc<tokio::sync::Mutex<SendStream>>
) {
    // let command = data.req_cmdgram.unwrap();
    // 此方法中需要对 token 进行验证
    slog::info!(btcmtools::LOGGER, "client login server {:?}", reqcmdgram);
    let mut ccp = cpm.lock().await;
    // 缓存client的信息
    ccp.put_client(reqcmdgram.sender().into(), reqcmdgram.deviceid(), stm.clone());
    // 必须clone
    // ccp.put_stream(stmid, stm.clone());

    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let cdg = CommandDataGram::create_command_gram_from_gram(vecu8.as_mut(), reqcmdgram.as_ref());

    let mut stream = stm.lock().await;
    // let mut bts = rescmdbuff.as_ref();
    let u8array = vecu8.as_mut();
    stream.write_all(u8array).await.expect("stream should be open");
    // stream.write_all(u8array).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}
