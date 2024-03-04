use std::{error::Error, time::Duration};
// use tokio::sync::Mutex;
use tokio::time::interval;

use crate::connservice::ClientPoolManager;

pub async fn start_watch_dog_server(
    // cpm0: Arc<Mutex<ClientPoolManager>>,
) -> Result<(), Box<dyn Error>> {
    // 定义清除超时键值对的时间间隔
    let tt = Duration::from_secs(10 * 60);

    // 创建定时器
    let mut timer = interval(tt);

    // 在循环中执行清除超时键值对的逻辑
    loop {
        timer.tick().await;
        // let mut ccp = cpm0.lock().await;
        ClientPoolManager::remove_expired().await;
    }
}
