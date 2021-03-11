## tetsy-libp2p-identify

Implementation of the [Identify] protocol.

This implementation of the protocol periodically exchanges
[`IdentifyInfo`] messages between the peers on an established connection.

At least one identification request is sent on a newly established
connection, beyond which the behaviour does not keep connections alive.

# Usage

The [`Identify`] struct implements a `NetworkBehaviour` that negotiates
and executes the protocol on every established connection, emitting
[`IdentifyEvent`]s.

[Identify]: https://github.com/libp2p/specs/tree/master/identify
[`Identify`]: self::Identify
[`IdentifyEvent`]: self::IdentifyEvent
[`IdentifyInfo`]: self::IdentifyEvent