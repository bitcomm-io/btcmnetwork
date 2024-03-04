use btcmbase::datagram::{ BitCommand, CommandDataGram };
use bytes::Bytes;
use s2n_quic::stream::SendStream;

use std::sync::Arc;
use crate::{ login, logout, pingpong };

/// 处理命令数据
///
/// 此函数根据命令数据报的类型，调用相应的处理函数来处理命令数据。
///
/// # Arguments
///
/// * `reqcmdbuff` - 命令缓冲区的 Arc<Bytes> 引用
/// * `reqcmdgram` - 命令数据报的 Arc<CommandDataGram> 引用
///
/// # Examples
///
/// ```rust
/// use btcmbase::datagram::{BitCommand, CommandDataGram};
/// use bytes::Bytes;
/// use std::sync::Arc;
/// use crate::{ login, logout, pingpong};
///
/// let reqcmdbuff = Arc::new(Bytes::new());
/// let reqcmdgram = Arc::new(CommandDataGram::default());
/// handle_command_data(&reqcmdbuff, &reqcmdgram);
/// ```
pub fn handle_command_data(reqcmdbuff: &Arc<Bytes>, reqcmdgram: &Arc<CommandDataGram>) {
    // 处理login命令
    if reqcmdgram.command().contains(BitCommand::LOGIN_COMMAND) {
        login::handle_command_login(reqcmdbuff, reqcmdgram)
    } else if reqcmdgram.command().contains(BitCommand::LOGOUT_COMMAND) {
        logout::handle_command_logout(reqcmdbuff, reqcmdgram)
    }
}

/// 异步处理命令数据
///
/// 此函数根据命令数据报的类型，调用相应的异步处理函数来处理命令数据。
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
/// use btcmbase::datagram::{BitCommand, CommandDataGram};
/// use bytes::Bytes;
/// use std::sync::Arc;
/// use s2n_quic::stream::SendStream;
/// use tokio::sync::Mutex;
/// use crate::{ login, logout, pingpong};
///
/// let reqcmdbuff = Arc::new(Bytes::new());
/// let reqcmdgram = Arc::new(CommandDataGram::default());
/// let stm = Arc::new(Mutex::new(SendStream::default()));
/// process_command_data(&reqcmdbuff, &reqcmdgram, stm).await;
/// ```
#[allow(unused_variables)]
pub async fn process_command_data<'a>(
    reqcmdbuff: &Arc<Bytes>,
    reqcmdgram: &Arc<CommandDataGram>,
    stm: Arc<tokio::sync::Mutex<SendStream>>
) {
    match reqcmdgram.command() {
        BitCommand::LOGIN_COMMAND =>
            login::process_command_login(reqcmdbuff, reqcmdgram, stm).await,
        BitCommand::LOGOUT_COMMAND =>
            logout::process_command_logout(reqcmdbuff, reqcmdgram, stm).await,
        BitCommand::SEND_PING =>
            pingpong::process_command_pingpong(reqcmdbuff, reqcmdgram, stm).await,
        _ => {}
    }
}

// use btcmbase::datagram::{BitCommand, CommandDataGram};
// use bytes::Bytes;
// #[allow(unused_imports)]
// use s2n_quic::{
//     stream::{BidirectionalStream, SendStream},
//     Server,
// };
// // use tokio::sync::{ mpsc::Sender, Mutex };
// // use tokio::sync::{mpsc::Sender, Mutex};

// use std::sync::Arc;
// // use tokio::sync::Mutex;
// use crate::{ login, logout, pingpong};

// //
// pub fn handle_command_data(
//     reqcmdbuff: &Arc<Bytes>,
//     reqcmdgram: &Arc<CommandDataGram>
// ) {
//     // 处理login命令
//     if reqcmdgram.command().contains(BitCommand::LOGIN_COMMAND) {
//         login::handle_command_login(reqcmdbuff, reqcmdgram)
//     } else if reqcmdgram.command().contains(BitCommand::LOGOUT_COMMAND) {
//         logout::handle_command_logout(reqcmdbuff, reqcmdgram)
//     }
// }
// #[allow(unused_variables)]
// pub async fn process_command_data<'a>(
//     reqcmdbuff: &Arc<Bytes>,
//     reqcmdgram: &Arc<CommandDataGram>,
//     stm: Arc<tokio::sync::Mutex<SendStream>>,
// ) {
//     match reqcmdgram.command() {
//         BitCommand::LOGIN_COMMAND   => login::process_command_login( reqcmdbuff, reqcmdgram, stm).await,
//         BitCommand::LOGOUT_COMMAND  => logout::process_command_logout( reqcmdbuff, reqcmdgram, stm).await,
//         BitCommand::SEND_PING       => pingpong::process_command_pingpong( reqcmdbuff, reqcmdgram, stm).await,
//         _ => {}
//     }
// }
