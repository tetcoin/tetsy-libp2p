## tetsy-libp2p-swarm

High level manager of the network.

A [`Swarm`] contains the state of the network as a whole. The entire
behaviour of a tetsy-libp2p network can be controlled through the `Swarm`.
The `Swarm` struct contains all active and pending connections to
remotes and manages the state of all the substreams that have been
opened, and all the upgrades that were built upon these substreams.

# Initializing a Swarm

Creating a `Swarm` requires three things:

 1. A network identity of the local node in form of a [`PeerId`].
 2. An implementation of the [`Transport`] trait. This is the type that
    will be used in order to reach nodes on the network based on their
    address. See the `transport` module for more information.
 3. An implementation of the [`NetworkBehaviour`] trait. This is a state
    machine that defines how the swarm should behave once it is connected
    to a node.

# Network Behaviour

The [`NetworkBehaviour`] trait is implemented on types that indicate to
the swarm how it should behave. This includes which protocols are supported
and which nodes to try to connect to. It is the `NetworkBehaviour` that
controls what happens on the network. Multiple types that implement
`NetworkBehaviour` can be composed into a single behaviour.

# Protocols Handler

The [`ProtocolsHandler`] trait defines how each active connection to a
remote should behave: how to handle incoming substreams, which protocols
are supported, when to open a new outbound substream, etc.
