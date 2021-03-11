## request-response

Generic request/response protocols.

## General Usage

[`RequestResponse`] is a `NetworkBehaviour` that implements a generic
request/response protocol or protocol family, whereby each request is
sent over a new substream on a connection. `RequestResponse` is generic
over the actual messages being sent, which are defined in terms of a
[`RequestResponseCodec`]. Creating a request/response protocol thus amounts
to providing an implementation of this trait which can then be
given to [`RequestResponse::new`]. Further configuration options are
available via the [`RequestResponseConfig`].

Requests are sent using [`RequestResponse::send_request`] and the
responses received as [`RequestResponseMessage::Response`] via
[`RequestResponseEvent::Message`].

Responses are sent using [`RequestResponse::send_response`] upon
receiving a [`RequestResponseMessage::Request`] via
[`RequestResponseEvent::Message`].

## Protocol Families

A single [`RequestResponse`] instance can be used with an entire
protocol family that share the same request and response types.
For that purpose, [`RequestResponseCodec::Protocol`] is typically
instantiated with a sum type.

## Limited Protocol Support

It is possible to only support inbound or outbound requests for
a particular protocol. This is achieved by instantiating `RequestResponse`
with protocols using [`ProtocolSupport::Inbound`] or
[`ProtocolSupport::Outbound`]. Any subset of protocols of a protocol
family can be configured in this way. Such protocols will not be
advertised during inbound respectively outbound protocol negotiation
on the substreams.