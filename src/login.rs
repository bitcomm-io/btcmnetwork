#[allow(unused_imports)]
use std::{sync::Arc, rc::Rc};

use btcmbase::datagram::CommandDataGram;
use bytes::Bytes;
#[allow(unused_imports)]
use s2n_quic::stream::SendStream;
use tokio::io::AsyncWriteExt;
// use tokio::io::AsyncWriteExt;

use crate::connservice::ClientPoolManager;
#[allow(unused_variables)]
pub fn handle_command_login(reqcmdbuff:&Arc<Bytes>,reqcmdgram:&Arc<CommandDataGram>) {

// let mut ve = res.res_cmdu8.as_ref().unwrap();
// let rescommand = CommandDataGram::create_command_gram_from_message_gram(ve.as_mut_slice(), command);

      // Option::Some(BitcommDataGram::new_command_data_gram(Option::Some(reqcmd), 
      // Option::Some(reqdata),
        // Option::Some(rescmd),
      // Option::Some(resdata)))
}

#[allow(unused_variables)]
pub async fn process_command_login<'a>(stmid   :u64,
                                      reqcmdbuff:&Arc<Bytes>,reqcmdgram:&Arc<CommandDataGram>,
                                      cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                                      stm     :Arc<tokio::sync::Mutex<SendStream>>) {
    // let command = data.req_cmdgram.unwrap();
    // 此方法中需要对 token 进行验证
    eprintln!("client login server {:?}", reqcmdgram);   
    // tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut ccp = cpm.lock().await;
        // 缓存client的信息
        ccp.put_client(reqcmdgram.sender().into(), reqcmdgram.deviceid(), stmid);
        // 必须clone
        ccp.put_stream(stmid, stm.clone());

        let mut vecu8 = CommandDataGram::create_gram_buf(0);
        let cdg = CommandDataGram::create_command_gram_from_gram(vecu8.as_mut(), reqcmdgram.as_ref());

        let mut stream = stm.lock().await;
        // let mut bts = rescmdbuff.as_ref();
        let u8array = vecu8.as_mut();
        stream.write_all(u8array).await.expect("stream should be open");
        // stream.write_all(u8array).await.expect("stream should be open");
        stream.flush().await.expect("stream should be open");
    // })
  }