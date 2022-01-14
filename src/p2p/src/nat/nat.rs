use std::net::IpAddr;
use std::time::Duration;
use crate::error::Error;
use crate::protocol::ProtocolId;

pub enum NatProtocol {
    UDP,
    TCP
}

/// An implementation of nat.Interface can map local ports to ports
/// accessible from the Internet.
pub trait Interface {
    /// These methods manage a mapping between a port on the local
    /// machine to a port that can be connected to from the internet.

    /// `protocol` is "UDP" or "TCP". Some implementations allow setting
    /// a display name for the mapping. The mapping may be removed by
    /// the gateway when its lifetime ends.
    fn add_mapping(&mut self, protocol: NatProtocol, ext_port: u64, int_port: u64, name: &str, lifetime: Duration) -> Result<(), Error>;

    fn delete_mapping(&mut self, protocol: NatProtocol, ext_port: u64, int_port: u64) -> Result<(), Error>;

    /// This method should return the external (Internet-facing)
    /// address of the gateway device.
    fn external_ip(&self) -> Result<IpAddr, Error>;
}