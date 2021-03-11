## wasm-ext

Implementation of the tetsy-libp2p `Transport` trait for external transports.

This `Transport` is used in the context of WASM to allow delegating the transport mechanism
to the code that uses tetsy-libp2p, as opposed to inside of tetsy-libp2p itself.

> **Note**: This only allows transports that produce a raw stream with the remote. You
>           couldn't, for example, pass an implementation QUIC.

# Usage

Call `new()` with a JavaScript object that implements the interface described in the `ffi`
module.
