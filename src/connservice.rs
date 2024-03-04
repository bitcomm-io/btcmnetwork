use std::{ collections::HashMap, sync::Arc, time::{ Duration, Instant } };
use btcmbase::client::{ DeviceConnInfo, DeviceConnState };
use s2n_quic::stream::SendStream;
use tokio::sync::Mutex;

/// 默认超时时间为 120 秒。
#[warn(dead_code)]
pub static _TIME_OUT_: Duration = Duration::from_secs(120);

use lazy_static::lazy_static;
lazy_static! {
    /// 客户端池，键为 ClientID + DeviceID，值为发送流和过期时间的元组。
    pub static ref CLIENT_POOL: Arc<Mutex<HashMap<u128, (Arc<Mutex<SendStream>>, Instant)>>> = Arc::new(Mutex::new(HashMap::new()));
    /// 设备池，键为 ClientID，值为设备ID和设备连接信息的哈希表。
    pub static ref DEVICE_POOL: Arc<Mutex<HashMap<u64, HashMap<u32, DeviceConnInfo>>>> = Arc::new(Mutex::new(HashMap::new()));
    /// IP 地址池，键为客户端地址，值为 ClientID + DeviceID。
    pub static ref IPADRS_POOL: Arc<Mutex<HashMap<String,u128>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug)]
pub struct ClientPoolManager;
/// 获取由 `clientid` 和 `deviceid` 组成的键。
/// 
/// # 参数
/// - `clientid`: 客户端ID。
/// - `deviceid`: 设备ID。
/// 
/// # 返回
/// 返回由 `clientid` 和 `deviceid` 组成的 `u128` 类型的键。
fn get_key(clientid: u64, deviceid: u32) -> u128 {
    ((clientid as u128) << 64) | (deviceid as u128)
}
//
#[allow(dead_code)]
/// 通过键获取 `clientid` 和 `deviceid`。
/// 
/// # 参数
/// - `key`: 由 `clientid` 和 `deviceid` 组成的 `u128` 类型的键。
/// 
/// # 返回
/// 返回元组，包含 `clientid` 和 `deviceid`。
pub fn get_cd_by_key(key: u128) -> (u64, u32) {
    let clientid = (key >> 64) as u64;
    let deviceid = key as u32;
    (clientid, deviceid)
}
impl ClientPoolManager {
    /// 获取指定客户端的设备池。
    /// 
    /// # 参数
    /// - `clt`: 客户端ID。
    /// 
    /// # 返回
    /// 返回指定客户端的设备池。
    pub async fn get_device_pool(clt: u64) -> Option<HashMap<u32, DeviceConnInfo>> {
        let device_pool = DEVICE_POOL.lock().await;
        device_pool.get(&clt).map(|x| x.clone())
    }

    /// 获取指定客户端的发送流。
    /// 
    /// # 参数
    /// - `clt`: 客户端ID。
    /// - `dev`: 设备ID。
    /// 
    /// # 返回
    /// 返回发送流的 `Arc<Mutex<SendStream>>`。
    pub async fn get_client(clt: u64, dev: u32) -> Option<Arc<Mutex<SendStream>>> {
        let client_pool = CLIENT_POOL.lock().await;
        client_pool.get(&get_key(clt, dev)).map(|(v, _)| v.clone())
    }
    /// 移除指定客户端的发送流，并将设备状态设置为离线。
    /// 
    /// # 参数
    /// - `clt`: 客户端ID。
    /// - `dev`: 设备ID。
    /// 
    /// # 返回
    /// 返回被移除的发送流的 `Arc<Mutex<SendStream>>`。
    pub async fn remove_client(clt: u64, dev: u32) -> Option<Arc<Mutex<SendStream>>> {
        let mut device_pool = DEVICE_POOL.lock().await;
        if let Some(hash) = device_pool.get_mut(&clt) {
            if let Some(devinfo) = hash.get_mut(&dev) {
                // 将指定设备设置为离线状态
                devinfo.set_device_state(DeviceConnState::STATE_OFFLINE);
            }
        }
        let mut client_pool = CLIENT_POOL.lock().await;
        client_pool.remove(&get_key(clt, dev)).map(|(v, _)| v)
    }
    /// 设置指定设备的连接状态。
    /// 
    /// # 参数
    /// - `clt`: 客户端ID。
    /// - `dev`: 设备ID。
    /// - `state`: 设备连接状态。
    pub async fn set_device_state( clt: u64, dev: u32, state: DeviceConnState) {
        let mut device_pool = DEVICE_POOL.lock().await;
        // 如果客户端不存在，则插入新的客户端，设备状态设置为在线
        let hash = device_pool.entry(clt).or_insert_with(HashMap::new);
        hash.entry(dev)
            .or_insert_with(|| DeviceConnInfo::new(dev, state))
            .set_device_state(state);
    }
    /// 将客户端地址映射到 ClientID + DeviceID。
    /// 
    /// # 参数
    /// - `address`: 客户端地址。
    /// - `clt`: 客户端ID。
    /// - `dev`: 设备ID。
    /// 
    /// # 返回
    /// 返回被插入的客户端地址对应的键值。
    pub async fn put_addres(
        address:String,
        clt:u64,
        dev:u32
    ) -> Option<u128> {
        let key = get_key(clt, dev);
        let mut ipadrs_pool = IPADRS_POOL.lock().await;
        ipadrs_pool.insert(address, key)
    }
    ///
    pub async fn get_addres(
        address:&String
    ) ->Option<u128> {
        let ipadrs_pool = IPADRS_POOL.lock().await;
        ipadrs_pool.get(address).copied() //.map(|v|{v.clone()})
    }
    /// 从地址池中移除指定地址。
    /// 
    /// # 参数
    /// - `address`: 要移除的地址。
    /// 
    /// # 返回
    /// 如果成功移除地址，则返回该地址对应的客户端ID和设备ID组成的 `u128` 类型值，否则返回 `None`。
    pub async fn remove_addres(
        address:&String,
    ) -> Option<u128> {
        let mut ipadrs_pool = IPADRS_POOL.lock().await;
        ipadrs_pool.remove(address)
    }
    /// 将客户端添加到客户端池。
    /// 
    /// # 参数
    /// - `clt`: 客户端ID。
    /// - `dev`: 设备ID。
    /// - `stream`: 发送流。
    /// 
    /// # 返回
    /// 返回被插入的发送流的 `Arc<Mutex<SendStream>>`。
    pub async fn put_client(
        clt: u64,
        dev: u32,
        stream: Arc<Mutex<SendStream>>
    ) -> Option<Arc<Mutex<SendStream>>> {
        // 如果还不存在,则插入新的,将设备状态设为ONLINE
        Self::set_device_state(clt, dev, DeviceConnState::STATE_ONLINE).await;

        let key = get_key(clt, dev);
        let mut client_pool = CLIENT_POOL.lock().await;
        // 如果已经存在
        // if client_pool.contains_key(&key) {
            // client_pool.remove(&get_key(clt, dev));//.map(|(v, _)| v);
            // remove_client(clt, dev);
        // }
        client_pool.insert(key, (stream, Instant::now() + _TIME_OUT_)).map(|(v, _)| v)
    }
    /// 更新客户端的超时时间。
    /// 
    /// # 参数
    /// - `clt`: 客户端ID。
    /// - `dev`: 设备ID。
    /// 
    /// # 返回
    /// 返回更新后的超时时间。
    pub async fn update_client(clt: u64, dev: u32) -> Option<Instant> {
        let key = get_key(clt, dev);
        let mut client_pool = CLIENT_POOL.lock().await;
        if let Some((_, instant)) = client_pool.get_mut(&key) {
            *instant = Instant::now() + _TIME_OUT_; // 修改 Instant 值为当前时间加上超时时间
            Some(*instant)
        } else {
            None
        }
    }
    /// 移除已过期的客户端。
    pub async fn remove_expired() {
        let now = Instant::now();
        let mut client_pool = CLIENT_POOL.lock().await;
        client_pool.retain(|_, (_, expires)| *expires > now);
    }
}

// clientid下可以有多个不同的设备登录
// 以clientid为key的hash表,value的值是list

// // client_pool如果已经在设备和流的hash,则直接使用
// if self.client_pool.contains_key(&clientid) {
//     let device_pool = self.client_pool.get_mut(&clientid).unwrap();
//     // 如果device_pool不存在,则插入
//     if !device_pool.contains_key(&deviceid) {
//         device_pool.insert(deviceid,stream);
//     } else { // 如果已经存在stream,则需要检查是不是同一个
//         let oldstream = device_pool.get(&deviceid).unwrap();
//         // 如果hash中的和传递过来的不是同一个
//         if !Arc::ptr_eq(&stream, oldstream) {
//             device_pool.remove(&deviceid); // 先删除原来的
//             device_pool.insert(deviceid, stream);
//             // oldstream.lock().await.connection().clone();
//         }
//     }
//     Option::Some(device_pool)
// } else { // 如果还不存在device_pool,则新建一个device_pool
//     let mut device_pool: HashMap<u32, Arc<tokio::sync::Mutex<SendStream>>> = HashMap::new();
//     device_pool.insert(deviceid,stream, );
//     self.client_pool.insert(clientid, device_pool);
//     self.client_pool.get(&clientid)
// }
