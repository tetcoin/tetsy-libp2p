## tetsy-libp2p-mdns

mDNS is a protocol defined by [RFC 6762](https://tools.ietf.org/html/rfc6762) that allows
querying nodes that correspond to a certain domain name.

In the context of tetsy-libp2p, the mDNS protocol is used to discover other nodes on the local
network that support tetsy-libp2p.

# Usage

This crate provides the `Mdns` struct which implements the `NetworkBehaviour` trait. This
struct will automatically discover other tetsy-libp2p nodes on the local network.
