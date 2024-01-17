use std::sync::Arc;

use btcmbase::datagram::{CommandDataGram, BitcommDataGram};
use bytes::Bytes;
#[allow(unused_imports)]
use s2n_quic::stream::{SendStream, BidirectionalStream};
use tokio::io::AsyncWriteExt;

use crate::connservice::ClientPoolManager;

pub fn handle_command_login<'a>(data:&Bytes,command:&'a CommandDataGram) -> Option<BitcommDataGram<'a>> {

    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let rescommand = CommandDataGram::create_command_gram_from_message_gram(vecu8.as_mut_slice(), command);
 
    Option::Some(BitcommDataGram::new_command_data_gram(Option::Some(command), 
                                                      Option::Some(data.as_ref()),
                                                        Option::Some(rescommand),
                                                      Option::Some(vecu8.as_slice()),))
}

#[allow(unused_variables)]
pub async fn process_command_login(command:&CommandDataGram,
                            stmid   :u64,
                            cpm     :Arc<tokio::sync::Mutex<ClientPoolManager>>,
                            stream  :Arc<tokio::sync::Mutex<BidirectionalStream>>) 
                            -> Option<bytes::Bytes> {
    // 此方法中需要对 token 进行验证
    eprintln!("client login server {:?}", command);   
    // 
    let mut ccp = cpm.lock().await;
    // 缓存client的信息
    ccp.put_client(command.sender().into(), command.deviceid(), stmid);
    // 必须clone
    // ccp.put_stream(stmid, stream);
    // 生成返回消息
    let mut vecu8 = CommandDataGram::create_gram_buf(0);
    let rescommand = CommandDataGram::create_command_gram_from_message_gram(vecu8.as_mut_slice(), command);
    // eprintln!("Stream opened gram    from {:?}", rescommand);

    let mut stm = stream.lock().await;
    stm.write_all(vecu8.as_ref()).await.expect("stream should be open");
    // stream.send(datagram).await.expect("stream should be open");
    stm.flush().await.expect("stream should be open");

    Option::Some(bytes::Bytes::from(vecu8)) 
}