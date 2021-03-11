## tetsy-libp2p-ping

This module implements the `/ipfs/ping/1.0.0` protocol.

The ping protocol can be used as a simple application-layer health check
for connections of any [`Transport`] as well as to measure and record
round-trip times.

# Usage

The [`Ping`] struct implements the [`NetworkBehaviour`] trait. When used with a [`Swarm`],
it will respond to inbound ping requests and as necessary periodically send outbound
ping requests on every established connection. If a configurable number of consecutive
pings fail, the connection will be closed.

The `Ping` network behaviour produces [`PingEvent`]s, which may be consumed from the `Swarm`
by an application, e.g. to collect statistics.

> **Note**: The ping protocol does not keep otherwise idle connections alive
> by default, see [`PingConfig::with_keep_alive`] for changing this behaviour.

[`Swarm`]: tetsy_libp2p_swarm::Swarm
[`Transport`]: tetsy_libp2p_core::Transport