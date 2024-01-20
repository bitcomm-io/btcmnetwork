#[allow(unused_imports)]
use std::{sync::Arc, rc::Rc};

use btcmbase::datagram::CommandDataGram;
use bytes::Bytes;
use s2n_quic::stream::SendStream;
#[allow(unused_imports)]
use tokio::io::AsyncWriteExt;

use crate::connservice::ClientPoolManager;
#[allow(unused_variables)]
pub fn handle_command_logout(reqcmdbuff:&Arc<Bytes>,reqcmdgram:&Arc<CommandDataGram>) {
// let mut ve = res.res_cmdu8.as_ref().unwrap();
// let rescommand = CommandDataGram::create_command_gram_from_message_gram(ve.as_mut_slice(), command);

  // Option::Some(BitcommDataGram::new_command_data_gram(Option::Some(reqcmd), 
  //                                                   Option::Some(reqdata),
  //                                                     Option::Some(rescmd),
  //                                                   Option::Some(resdata),))
}

#[allow(unused_variables)]
pub fn process_command_logout<'a>(stmid   :u64,
                                  reqcmdbuff:&Arc<Bytes>,reqcmdgram:&Arc<CommandDataGram>,
                                  cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                                  stm     :Arc<tokio::sync::Mutex<SendStream>>) {
      // let command = data.req_cmdgram.unwrap();
      // 此方法中需要对 token 进行验证
      eprintln!("client login server {:?}", reqcmdgram);   
      // 
      tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut ccp = cpm.lock().await;
        // 缓存client的信息
        ccp.remove_client(reqcmdgram.sender().into(), reqcmdgram.deviceid());
        ccp.remove_stream(stmid);

        let stream = stm.lock().await;
        // let mut bts = rescmdbuff.as_ref();
        // let u8array = bts.as_mut();
        // // rescmdgram.set_bitcomm(BitcommFlag::BITCOMM_COMMAND);
        // // stream.write_all()
        // stream.write_all(u8array).await.expect("stream should be open");
        // stream.flush().await.expect("stream should be open");
    })
} 