## tetsy-libp2p-core

Transports, upgrades, multiplexing and node handling of *tetsy-libp2p*.

The main concepts of tetsy-libp2p-core are:

- A [`PeerId`] is a unique global identifier for a node on the network.
  Each node must have a different [`PeerId`]. Normally, a [`PeerId`] is the
  hash of the public key used to negotiate encryption on the
  communication channel, thereby guaranteeing that they cannot be spoofed.
- The [`Transport`] trait defines how to reach a remote node or listen for
  incoming remote connections. See the [`transport`] module.
- The [`StreamMuxer`] trait is implemented on structs that hold a connection
  to a remote and can subdivide this connection into multiple substreams.
  See the [`muxing`] module.
- The [`UpgradeInfo`], [`InboundUpgrade`] and [`OutboundUpgrade`] traits
  define how to upgrade each individual substream to use a protocol.
  See the `upgrade` module.