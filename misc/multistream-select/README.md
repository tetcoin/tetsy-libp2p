## Tetsy Multistream-select Protocol Negotiation

This crate implements the `tetsy-multistream-select` protocol, which is the protocol
used by tetsy-libp2p to negotiate which application-layer protocol to use with the
remote on a connection or substream.

> **Note**: This crate is used primarily by core components of *tetsy-libp2p* and it
> is usually not used directly on its own.

## Roles

Two peers using the tetsy-multistream-select negotiation protocol on an I/O stream
are distinguished by their role as a _dialer_ (or _initiator_) or as a _listener_
(or _responder_). Thereby the dialer plays the active part, driving the protocol,
whereas the listener reacts to the messages received.

The dialer has two options: it can either pick a protocol from the complete list
of protocols that the listener supports, or it can directly suggest a protocol.
Either way, a selected protocol is sent to the listener who can either accept (by
echoing the same protocol) or reject (by responding with a message stating
"not available"). If a suggested protocol is not available, the dialer may
suggest another protocol. This process continues until a protocol is agreed upon,
yielding a [`Negotiated`](self::Negotiated) stream, or the dialer has run out of
alternatives.

See [`dialer_select_proto`](self::dialer_select_proto) and
[`listener_select_proto`](self::listener_select_proto).

## [`Negotiated`](self::Negotiated)

A `Negotiated` represents an I/O stream that has settled on a protocol
to use. By default, with [`Version::V1`], protocol negotiation is always
at least one dedicated round-trip message exchange, before application
data for the negotiated protocol can be sent by the dialer. There is
a variant [`Version::V1Lazy`] that permits 0-RTT negotiation if the
dialer only supports a single protocol. In that case, when a dialer
settles on a protocol to use, the [`DialerSelectFuture`] yields a
[`Negotiated`](self::Negotiated) I/O stream before the negotiation
data has been flushed. It is then expecting confirmation for that protocol
as the first messages read from the stream. This behaviour allows the dialer
to immediately send data relating to the negotiated protocol together with the
remaining negotiation message(s). Note, however, that a dialer that performs
multiple 0-RTT negotiations in sequence for different protocols layered on
top of each other may trigger undesirable behaviour for a listener not
supporting one of the intermediate protocols. See
[`dialer_select_proto`](self::dialer_select_proto) and the documentation
of [`Version::V1Lazy`] for further details.

## Examples

For a dialer:

```no_run
use async_std::net::TcpStream;
use tetsy_multistream_select::{dialer_select_proto, Version};
use futures::prelude::*;

async_std::task::block_on(async move {
    let socket = TcpStream::connect("127.0.0.1:10333").await.unwrap();

    let protos = vec![b"/echo/1.0.0", b"/echo/2.5.0"];
    let (protocol, _io) = dialer_select_proto(socket, protos, Version::V1).await.unwrap();

    println!("Negotiated protocol: {:?}", protocol);
    // You can now use `_io` to communicate with the remote.
});
```
