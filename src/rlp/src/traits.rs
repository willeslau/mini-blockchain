// use crate::error::Error;
// use crate::rlp::RPLStream;
//
// /// RPL encodable trait. Encode Self into bytes and append to end of stream.
// pub trait Encodable {
//     fn encode(&self, stream: &mut RPLStream) -> Result<(), Error>;
// }
//
// /// RPL decodable trait. Decode from the stream to Self. Read from start of stream.
// pub trait Decodable {
//     fn decode(&self, stream: &mut RPLStream) -> Result<Self, Error> where Self: Sized;
// }
//
// impl Encodable for u8 {
//     fn encode(&self, stream: &mut RPLStream) -> Result<(), Error> {
//         todo!()
//     }
// }