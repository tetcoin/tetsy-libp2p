## tetsy-libp2p-relay

Implementation of the [libp2p circuit relay
specification](https://github.com/libp2p/specs/tree/master/relay).

## Example

```rust
# use tetsy_libp2p_core::transport::memory::MemoryTransport;
# use ltetsy_ibp2p_relay::{RelayConfig, new_transport_and_behaviour};
# use tetsy_libp2p_swarm::Swarm;
# use tetsy_libp2p_core::{identity, Multiaddr, multiaddr::Protocol, PeerId, upgrade, Transport};
# use libp2p_remux::RemuxConfig;
# use plaintext::PlainText2Config;
# use std::convert::TryInto;
# use std::str::FromStr;
#
# let local_key = identity::Keypair::generate_ed25519();
# let local_public_key = local_key.public();
# let local_peer_id = local_public_key.clone().into_peer_id();
# let plain = PlainText2Config {
#     local_public_key: local_public_key.clone(),
# };
#
let (relay_transport, relay_behaviour) = new_transport_and_behaviour(
    RelayConfig::default(),
    MemoryTransport::default(),
);

let transport = relay_transport
    .upgrade(upgrade::Version::V1)
    .authenticate(plain)
    .multiplex(RemuxConfig::default())
    .boxed();

let mut swarm = Swarm::new(transport, relay_behaviour, local_peer_id);

let relay_addr = Multiaddr::from_str("/memory/1234").unwrap()
    .with(Protocol::P2p(PeerId::random().into()))
    .with(Protocol::P2pCircuit);
let dst_addr = relay_addr.clone().with(Protocol::Memory(5678));

// Listen for incoming connections via relay node (1234).
Swarm::listen_on(&mut swarm, relay_addr).unwrap();

// Dial node (5678) via relay node (1234).
Swarm::dial_addr(&mut swarm, dst_addr).unwrap();
```

## Terminology

### Entities

- **Source**: The node initiating a connection via a *relay* to a *destination*.

- **Relay**: The node being asked by a *source* to relay to a *destination*.

- **Destination**: The node contacted by the *source* via the *relay*.

### Messages

- **Outgoing relay request**: The request sent by a *source* to a *relay*.

- **Incoming relay request**: The request received by a *relay* from a *source*.

- **Outgoing destination request**: The request sent by a *relay* to a *destination*.

- **Incoming destination request**: The request received by a *destination* from a *relay*.