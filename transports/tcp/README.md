## tetsy-libp2p-tcp

Implementation of the tetsy-libp2p `Transport` trait for TCP/IP.

# Usage

This crate provides a `TcpConfig` and `TokioTcpConfig`, depending on
the enabled features, which implement the `Transport` trait for use as a
transport with `tetsy-libp2p-core` or `tetsy-libp2p-swarm`.