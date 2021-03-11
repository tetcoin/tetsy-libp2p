## tetsy-libp2p-noise

[Noise protocol framework][noise] support for tetsy-libp2p.

> **Note**: This crate is still experimental and subject to major breaking changes
>           both on the API and the wire protocol.

This crate provides `tetsy_libp2p_core::InboundUpgrade` and `tetsy_libp2p_core::OutboundUpgrade`
implementations for various noise handshake patterns (currently `IK`, `IX`, and `XX`)
over a particular choice of Diffie–Hellman key agreement (currently only X25519).

> **Note**: Only the `XX` handshake pattern is currently guaranteed to provide
>           interoperability with other libp2p implementations.

All upgrades produce as output a pair, consisting of the remote's static public key
and a `NoiseOutput` which represents the established cryptographic session with the
remote, implementing `futures::io::AsyncRead` and `futures::io::AsyncWrite`.

# Usage

Example:

```
use tetsy_libp2p_core::{identity, Transport, upgrade};
use tetsy_libp2p_tcp::TcpConfig;
use tetsy_libp2p_noise::{Keypair, X25519Spec, NoiseConfig};

# fn main() {
let id_keys = identity::Keypair::generate_ed25519();
let dh_keys = Keypair::<X25519Spec>::new().into_authentic(&id_keys).unwrap();
let noise = NoiseConfig::xx(dh_keys).into_authenticated();
let builder = TcpConfig::new().upgrade(upgrade::Version::V1).authenticate(noise);
// let transport = builder.multiplex(...);
# }
```

[noise]: http://noiseprotocol.org/