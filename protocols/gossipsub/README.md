## tetsy-libp2p-gossipsub

Gossipsub is a P2P pubsub (publish/subscription) routing layer designed to extend upon
floodsub and meshsub routing protocols.

# Overview

*Note: The gossipsub protocol specifications
(https://github.com/libp2p/specs/tree/master/pubsub/gossipsub) provide an outline for the
routing protocol. They should be consulted for further detail.*

Gossipsub  is a blend of meshsub for data and randomsub for mesh metadata. It provides bounded
degree and amplification factor with the meshsub construction and augments it using gossip
propagation of metadata with the randomsub technique.

The router maintains an overlay mesh network of peers on which to efficiently send messages and
metadata.  Peers use control messages to broadcast and request known messages and
subscribe/unsubscribe from topics in the mesh network.

# Important Discrepancies

This section outlines the current implementation's potential discrepancies from that of other
implementations, due to undefined elements in the current specification.

- **Topics** -  In gossipsub, topics configurable by the `hash_topics` configuration parameter.
Topics are of type [`TopicHash`]. The current go implementation uses raw utf-8 strings, and this
is default configuration in tetsy-libp2p. Topics can be hashed (SHA256 hashed then base64
encoded) by setting the `hash_topics` configuration parameter to true.

- **Sequence Numbers** - A message on the gossipsub network is identified by the source
[`tetsy_libp2p_core::PeerId`] and a nonce (sequence number) of the message. The sequence numbers in
this implementation are sent as raw bytes across the wire. They are 64-bit big-endian unsigned
integers. They are chosen at random in this implementation of gossipsub, but are sequential in
the current go implementation.

# Using Gossipsub

## GossipsubConfig

The [`GossipsubConfig`] struct specifies various network performance/tuning configuration
parameters. Specifically it specifies:

[`GossipsubConfig`]: struct.Config.html

This struct implements the [`Default`] trait and can be initialised via
[`GossipsubConfig::default()`].


## Gossipsub

The [`Gossipsub`] struct implements the [`tetsy_libp2p_swarm::NetworkBehaviour`] trait allowing it to
act as the routing behaviour in a [`tetsy_libp2p_swarm::Swarm`]. This struct requires an instance of
[`tetsy_libp2p_core::PeerId`] and [`GossipsubConfig`].

[`Gossipsub`]: struct.Gossipsub.html

## Example

An example of initialising a gossipsub compatible swarm:

```
use tetsy_libp2p_gossipsub::GossipsubEvent;
use tetsy_libp2p_core::{identity::Keypair,transport::{Transport, MemoryTransport}, Multiaddr};
use tetsy_libp2p_gossipsub::MessageAuthenticity;
let local_key = Keypair::generate_ed25519();
let local_peer_id = tetsy_libp2p_core::PeerId::from(local_key.public());

// Set up an encrypted TCP Transport over the Mplex
// This is test transport (memory).
let noise_keys = tetsy_libp2p_noise::Keypair::<tetsy_libp2p_noise::X25519Spec>::new().into_authentic(&local_key).unwrap();
let transport = MemoryTransport::default()
           .upgrade(tetsy_libp2p_core::upgrade::Version::V1)
           .authenticate(tetsy_libp2p_noise::NoiseConfig::xx(noise_keys).into_authenticated())
           .multiplex(tetsy_libp2p_mplex::MplexConfig::new())
           .boxed();

// Create a Gossipsub topic
let topic = tetsy_libp2p_gossipsub::IdentTopic::new("example");

// Set the message authenticity - How we expect to publish messages
// Here we expect the publisher to sign the message with their key.
let message_authenticity = MessageAuthenticity::Signed(local_key);

// Create a Swarm to manage peers and events
let mut swarm = {
    // set default parameters for gossipsub
    let gossipsub_config = tetsy_libp2p_gossipsub::GossipsubConfig::default();
    // build a gossipsub network behaviour
    let mut gossipsub: tetsy_libp2p_gossipsub::Gossipsub =
        tetsy_libp2p_gossipsub::Gossipsub::new(message_authenticity, gossipsub_config).unwrap();
    // subscribe to the topic
    gossipsub.subscribe(&topic);
    // create the swarm
    tetsy_libp2p_swarm::Swarm::new(
        transport,
        gossipsub,
        local_peer_id,
    )
};

// Listen on a memory transport.
let memory: Multiaddr = tetsy_libp2p_core::multiaddr::Protocol::Memory(10).into();
let addr = tetsy_libp2p_swarm::Swarm::listen_on(&mut swarm, memory).unwrap();
println!("Listening on {:?}", addr);
```