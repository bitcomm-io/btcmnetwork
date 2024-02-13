use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};
use s2n_quic::stream::SendStream;
use tokio::sync::Mutex;

// 默认120秒超时
#[warn(dead_code)]
pub static _TIME_OUT_ :Duration = Duration::from_secs(120);

#[derive(Debug)]
pub struct DeviceInfo {
    pub device_id       :u32,
    pub device_type     :u32,
    pub device_state    :u32,
}
#[derive(Debug)]
pub struct ClientPoolManager {
    /// u128 = clientid + deviceid
    client_pool : HashMap<u128,(Arc<Mutex<SendStream>>,Instant)>, // clientid,deviceid,streamid
    device_pool : HashMap<u64,Vec<u32>>,
    // 
    // stream_pool : HashMap<u128,Arc<tokio::sync::Mutex<SendStream>>>, // streamid,stream
}
fn get_key(clientid:u64,deviceid:u32) -> u128 {
    (clientid as u128) << 64 | (deviceid as u128)
}

impl ClientPoolManager {
    // 
    pub fn new() -> Self {
        ClientPoolManager { client_pool   : HashMap::new(),
                            device_pool   : HashMap::new(), 
                          }
    }
    // 
    pub fn get_device_pool(&self,clt:u64) -> Option<&Vec<u32>> {
        self.device_pool.get(&clt)
    }

    //
    pub fn get_client(&self,clt:u64,dev:u32) -> Option<&Arc<Mutex<SendStream>>> {
        self.client_pool.get(&get_key(clt,dev)).map(|(v, _)| v)
    }
    //
    pub fn remove_client(&mut self,clt:u64,dev:u32) -> Option<Arc<Mutex<SendStream>>> {

        if self.device_pool.contains_key(&clt) {
            let array = self.device_pool.get_mut(&clt).unwrap();
            array.retain(|&x| x != dev);
        }

        // if self.client_pool.contains_key(&key) {
        self.client_pool.remove(&get_key(clt,dev)).map(|(v, _)| v)
        // } else {
            // Option::None
        // }
    }
    //
    pub fn put_client(&mut self,clt:u64,dev:u32,stream:Arc<Mutex<SendStream>>) ->Option<Arc<Mutex<SendStream>>> {
        // 如果还不存在,则插入新的
        self.device_pool.entry(clt).or_insert(Vec::new());
        let array = self.device_pool.get_mut(&clt).unwrap();
        if !array.contains(&dev) {array.push(dev);}

        let key = get_key(clt,dev);
        // 如果已经存在
        if self.client_pool.contains_key(&key) {
            self.remove_client(clt,dev);
        }
        self.client_pool.insert(key, (stream, Instant::now() + _TIME_OUT_)).map(|(v, _)| v)
    }
    // 
    pub fn update_client(&mut self,clt:u64,dev:u32) ->Option<&Instant> {
        let key = get_key(clt,dev);
        // 如果存在,则需要修改超时时间
        if self.client_pool.contains_key(&key) {
            if let Some( (_, instant)) = self.client_pool.get_mut(&key) {
                *instant = Instant::now() + _TIME_OUT_; // 修改 Instant 值为当前时间
                Some(instant)
            } else {
                None
            }
        } else {
            None
        }
    }
    //
    pub fn remove_expired(&mut self) {
        let now = Instant::now();
        self.client_pool.retain(|_, (_, expires)| *expires > now);
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
