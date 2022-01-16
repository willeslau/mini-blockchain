use std::fmt::Error;
use crate::peer::PeerId;

pub type ProtocolId = u64;

#[derive(Debug, PartialEq, Eq)]
/// Protocol info
pub struct CapabilityInfo {
    /// Protocol ID
    pub protocol: ProtocolId,
    /// Protocol version
    pub version: u8,
    /// Total number of packet IDs this protocol support.
    pub packet_count: u8,
}

/// Protocol represents a P2P subprotocol implementation.
pub trait Protocol {
    /// Returns the id of the protocol
    fn id(&self) -> ProtocolId;
    /// Name should contain the official protocol name, often a three-letter word.
    fn name(&self) -> String;
    /// Version should contain the version number of the protocol.
    fn version(&self) -> u8;
    /// Length should contain the number of message codes used by the protocol.
    fn length(&self) -> u64;
    /// Run is called in a new goroutine when the protocol has been
    /// negotiated with a peer. It should read and write messages from
    /// rw. The Payload for each message must be fully consumed.
    /// The peer connection is closed when Start returns. It should return
    /// any protocol-level error (such as an I/O error) that is
    /// encountered.
    fn run(&self, peer: PeerId) -> Result<(), Error>;
    // fn run(peer: PeerId, rw MsgReadWriter) error

    // // NodeInfo is an optional helper method to retrieve protocol specific metadata
    // // about the host node.
    // NodeInfo func() interface{}
    //
    // // PeerInfo is an optional helper method to retrieve protocol specific metadata
    // // about a certain peer in the network. If an info retrieval function is set,
    // // but returns nil, it is assumed that the protocol handshake is still running.
    // PeerInfo func(id enode.ID) interface{}
    //
    // // DialCandidates, if non-nil, is a way to tell Server about protocol-specific nodes
    // // that should be dialed. The server continuously reads nodes from the iterator and
    // // attempts to create connections to them.
    // DialCandidates enode.Iterator
    //
    // // Attributes contains protocol specific information for the node record.
    // Attributes []enr.Entry
}