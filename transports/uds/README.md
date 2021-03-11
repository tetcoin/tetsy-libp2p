## tetsy-libp2p-uds

Implementation of the tetsy-libp2p `Transport` trait for Unix domain sockets.

# Platform support

This transport only works on Unix platforms.

# Usage

The `UdsConfig` transport supports multiaddresses of the form `/unix//tmp/foo`.

The `UdsConfig` structs implements the `Transport` trait of the `core` library. See the
documentation of `core` and of tetsy-libp2p in general to learn how to use the `Transport` trait.