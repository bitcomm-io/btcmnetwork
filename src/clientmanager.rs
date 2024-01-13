
use btcmbase::client::ClientID;
use getset::{Getters,Setters};
use s2n_quic::{Connection, stream::BidirectionalStream};
// use uuid::Uuid;



#[repr(C)] 
#[derive(Debug,Getters, Setters)]
pub struct ClientNetContext<'a> {
    pub clientid    : ClientID,
    pub deviceid    : String    ,
    pub connection  : &'a Connection,
    pub stream      : &'a BidirectionalStream,
}