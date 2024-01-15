use std::{collections::HashMap, sync::Arc};
use s2n_quic::stream::BidirectionalStream;


#[derive(Debug)]
pub struct ClientPoolManager {
    ///
    client_pool : HashMap<u64,HashMap<u64,u64>>, // clientid,streamid,deviceid
    /// 
    stream_pool : HashMap<u64,Arc<tokio::sync::Mutex<BidirectionalStream>>>, // streamid,stream
}

impl ClientPoolManager {

    pub fn new() -> Self {
        ClientPoolManager { client_pool : HashMap::new(),stream_pool : HashMap::new(), }
    }

    pub fn get_stream(&self,stream_id:u64) -> Option<&Arc<tokio::sync::Mutex<BidirectionalStream>>> {
        self.stream_pool.get(&stream_id)
    } 


    pub fn put_stream<'a>(&mut self,stream_id:u64,value:Arc<tokio::sync::Mutex<BidirectionalStream>>) -> Option<Arc<tokio::sync::Mutex<BidirectionalStream>>> {
        self.stream_pool.insert(stream_id, value)
    }


    pub fn get_client(&self,clientid:u64) -> Option<&HashMap<u64,u64>> {
        self.client_pool.get(&clientid)
    }


    pub fn put_client(&mut self,clientid:u64,deviceid:u64,streamid:u64) ->Option<&HashMap<u64,u64>> {
        // client_pool如果已经在设备和流的hash,则直接使用
        if self.client_pool.contains_key(&clientid) {
            let device_pool = self.client_pool.get_mut(&clientid).unwrap();
            // 如果device_pool不存在,则插入
            if !device_pool.contains_key(&streamid) {
                device_pool.insert(streamid,deviceid);
            }
            Option::Some(device_pool)
        } else { // 如果还不存在device_pool,则新建一个device_pool
            let mut device_pool: HashMap<u64, u64> = HashMap::new();
            device_pool.insert(streamid, deviceid);
            self.client_pool.insert(clientid, device_pool);
            self.client_pool.get(&clientid)
        }

    }
}

// clientid下可以有多个不同的设备登录
// 以clientid为key的hash表,value的值是list

