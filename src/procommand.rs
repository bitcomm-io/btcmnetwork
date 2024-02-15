use btcmbase::datagram::{BitCommand, CommandDataGram};
use bytes::Bytes;
#[allow(unused_imports)]
use s2n_quic::{
    stream::{BidirectionalStream, SendStream},
    Server,
};
use tokio::sync::{ mpsc::Sender, Mutex };
// use tokio::sync::{mpsc::Sender, Mutex};

use std::sync::Arc;
// use tokio::sync::Mutex;
use crate::{connservice::ClientPoolManager, eventqueue::MessageEvent, login, logout, pingpong};

//
pub fn handle_command_data<'a>(reqcmdbuff: &Arc<Bytes>, reqcmdgram: &Arc<CommandDataGram>) {
    // 处理login命令
    if reqcmdgram.command().contains(BitCommand::LOGIN_COMMAND) {
        login::handle_command_login(reqcmdbuff, reqcmdgram)
    } else if reqcmdgram.command().contains(BitCommand::LOGOUT_COMMAND) {
        logout::handle_command_logout(reqcmdbuff, reqcmdgram)
    }
}
#[allow(unused_variables)]
pub async fn process_command_data<'a>(
    stmid: u64,
    reqcmdbuff: &Arc<Bytes>,
    reqcmdgram: &Arc<CommandDataGram>,
    cpm: Arc<tokio::sync::Mutex<ClientPoolManager>>,
    stm: Arc<tokio::sync::Mutex<SendStream>>,
    meqsend: Arc<Mutex<Sender<MessageEvent>>>
) {
    match reqcmdgram.command() {
        BitCommand::LOGIN_COMMAND   => login::process_command_login(stmid, reqcmdbuff, reqcmdgram, cpm, stm).await,
        BitCommand::LOGOUT_COMMAND  => logout::process_command_logout(stmid, reqcmdbuff, reqcmdgram, cpm, stm).await,
        BitCommand::SEND_PING       => pingpong::process_command_pingpong(stmid, reqcmdbuff, reqcmdgram, cpm, stm).await,
        _ => {}
    }
}
