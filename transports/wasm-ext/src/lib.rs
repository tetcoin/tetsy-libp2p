// Copyright 2019 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! Implementation of the tetsy-libp2p `Transport` trait for external transports.
//!
//! This `Transport` is used in the context of WASM to allow delegating the transport mechanism
//! to the code that uses rust-libp2p, as opposed to inside of rust-tetsy-libp2p itself.
//!
//! > **Note**: This only allows transports that produce a raw stream with the remote. You
//! >           couldn't, for example, pass an implementation QUIC.
//!
//! # Usage
//!
//! Call `new()` with a JavaScript object that implements the interface described in the `ffi`
//! module.
//!

use futures::{prelude::*, future::Ready};
use tetsy_libp2p_core::{transport::ListenerEvent, transport::TransportError, Multiaddr, Transport};
use parity_send_wrapper::SendWrapper;
use std::{collections::VecDeque, error, fmt, io, mem, pin::Pin, task::Context, task::Poll};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;

/// Contains the definition that one must match on the JavaScript side.
pub mod ffi {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        /// Type of the object that allows opening connections.
        pub type Transport;
        /// Type of the object that represents an open connection with a remote.
        pub type Connection;
        /// Type of the object that represents an event generated by listening.
        pub type ListenEvent;
        /// Type of the object that represents an event containing a new connection with a remote.
        pub type ConnectionEvent;

        /// Start attempting to dial the given multiaddress.
        ///
        /// The returned `Promise` must yield a [`Connection`] on success.
        ///
        /// If the multiaddress is not supported, you should return an instance of `Error` whose
        /// `name` property has been set to the string `"NotSupportedError"`.
        #[wasm_bindgen(method, catch)]
        pub fn dial(this: &Transport, multiaddr: &str) -> Result<js_sys::Promise, JsValue>;

        /// Start listening on the given multiaddress.
        ///
        /// The returned `Iterator` must yield `Promise`s to [`ListenEvent`] events.
        ///
        /// If the multiaddress is not supported, you should return an instance of `Error` whose
        /// `name` property has been set to the string `"NotSupportedError"`.
        #[wasm_bindgen(method, catch)]
        pub fn listen_on(this: &Transport, multiaddr: &str) -> Result<js_sys::Iterator, JsValue>;

        /// Returns an iterator of JavaScript `Promise`s that resolve to `ArrayBuffer` objects
        /// (or resolve to null, see below). These `ArrayBuffer` objects contain the data that the
        /// remote has sent to us. If the remote closes the connection, the iterator must produce
        /// a `Promise` that resolves to `null`.
        #[wasm_bindgen(method, getter)]
        pub fn read(this: &Connection) -> js_sys::Iterator;

        /// Writes data to the connection. Returns a `Promise` that resolves when the connection is
        /// ready for writing again.
        ///
        /// If the `Promise` produces an error, the writing side of the connection is considered
        /// unrecoverable and the connection should be closed as soon as possible.
        ///
        /// Guaranteed to only be called after the previous write promise has resolved.
        #[wasm_bindgen(method, catch)]
        pub fn write(this: &Connection, data: &[u8]) -> Result<js_sys::Promise, JsValue>;

        /// Shuts down the writing side of the connection. After this has been called, the `write`
        /// method will no longer be called.
        #[wasm_bindgen(method, catch)]
        pub fn shutdown(this: &Connection) -> Result<(), JsValue>;

        /// Closes the connection. No other method will be called on this connection anymore.
        #[wasm_bindgen(method)]
        pub fn close(this: &Connection);

        /// List of addresses we have started listening on. Must be an array of strings of
        /// multiaddrs.
        #[wasm_bindgen(method, getter)]
        pub fn new_addrs(this: &ListenEvent) -> Option<Box<[JsValue]>>;

        /// List of addresses that have expired. Must be an array of strings of multiaddrs.
        #[wasm_bindgen(method, getter)]
        pub fn expired_addrs(this: &ListenEvent) -> Option<Box<[JsValue]>>;

        /// List of [`ConnectionEvent`] object that has been received.
        #[wasm_bindgen(method, getter)]
        pub fn new_connections(this: &ListenEvent) -> Option<Box<[JsValue]>>;

        /// Promise to the next event that the listener will generate.
        #[wasm_bindgen(method, getter)]
        pub fn next_event(this: &ListenEvent) -> JsValue;

        /// The [`Connection`] object for communication with the remote.
        #[wasm_bindgen(method, getter)]
        pub fn connection(this: &ConnectionEvent) -> Connection;

        /// The address we observe for the remote connection.
        #[wasm_bindgen(method, getter)]
        pub fn observed_addr(this: &ConnectionEvent) -> String;

        /// The address we are listening on, that received the remote connection.
        #[wasm_bindgen(method, getter)]
        pub fn local_addr(this: &ConnectionEvent) -> String;
    }

    #[cfg(feature = "websocket")]
    #[wasm_bindgen(module = "/src/websockets.js")]
    extern "C" {
        /// Returns a `Transport` implemented using websockets.
        pub fn websocket_transport() -> Transport;
    }
}

/// Implementation of `Transport` whose implementation is handled by some FFI.
pub struct ExtTransport {
    inner: SendWrapper<ffi::Transport>,
}

impl ExtTransport {
    /// Creates a new `ExtTransport` that uses the given external `Transport`.
    pub fn new(transport: ffi::Transport) -> Self {
        ExtTransport {
            inner: SendWrapper::new(transport),
        }
    }
}

impl fmt::Debug for ExtTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ExtTransport").finish()
    }
}

impl Clone for ExtTransport {
    fn clone(&self) -> Self {
        ExtTransport {
            inner: SendWrapper::new(self.inner.clone().into()),
        }
    }
}

impl Transport for ExtTransport {
    type Output = Connection;
    type Error = JsErr;
    type Listener = Listen;
    type ListenerUpgrade = Ready<Result<Self::Output, Self::Error>>;
    type Dial = Dial;

    fn listen_on(self, addr: Multiaddr) -> Result<Self::Listener, TransportError<Self::Error>> {
        let iter = self
            .inner
            .listen_on(&addr.to_string())
            .map_err(|err| {
                if is_not_supported_error(&err) {
                    TransportError::MultiaddrNotSupported(addr)
                } else {
                    TransportError::Other(JsErr::from(err))
                }
            })?;

        Ok(Listen {
            iterator: SendWrapper::new(iter),
            next_event: None,
            pending_events: VecDeque::new(),
        })
    }

    fn dial(self, addr: Multiaddr) -> Result<Self::Dial, TransportError<Self::Error>> {
        let promise = self
            .inner
            .dial(&addr.to_string())
            .map_err(|err| {
                if is_not_supported_error(&err) {
                    TransportError::MultiaddrNotSupported(addr)
                } else {
                    TransportError::Other(JsErr::from(err))
                }
            })?;

        Ok(Dial {
            inner: SendWrapper::new(promise.into()),
        })
    }

    fn address_translation(&self, _server: &Multiaddr, _observed: &Multiaddr) -> Option<Multiaddr> {
        None
    }
}

/// Future that dial a remote through an external transport.
#[must_use = "futures do nothing unless polled"]
pub struct Dial {
    /// A promise that will resolve to a `ffi::Connection` on success.
    inner: SendWrapper<JsFuture>,
}

impl fmt::Debug for Dial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Dial").finish()
    }
}

impl Future for Dial {
    type Output = Result<Connection, JsErr>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Future::poll(Pin::new(&mut *self.inner), cx) {
            Poll::Ready(Ok(connec)) => Poll::Ready(Ok(Connection::new(connec.into()))),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => Poll::Ready(Err(JsErr::from(err))),
        }
    }
}

/// Stream that listens for incoming connections through an external transport.
#[must_use = "futures do nothing unless polled"]
pub struct Listen {
    /// Iterator of `ListenEvent`s.
    iterator: SendWrapper<js_sys::Iterator>,
    /// Promise that will yield the next `ListenEvent`.
    next_event: Option<SendWrapper<JsFuture>>,
    /// List of events that we are waiting to propagate.
    pending_events: VecDeque<ListenerEvent<Ready<Result<Connection, JsErr>>, JsErr>>,
}

impl fmt::Debug for Listen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Listen").finish()
    }
}

impl Stream for Listen {
    type Item = Result<ListenerEvent<Ready<Result<Connection, JsErr>>, JsErr>, JsErr>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(ev) = self.pending_events.pop_front() {
                return Poll::Ready(Some(Ok(ev)));
            }

            // Try to fill `self.next_event` if necessary and possible. If we fail, then
            // `Ready(None)` is returned below.
            if self.next_event.is_none() {
                if let Ok(ev) = self.iterator.next() {
                    if !ev.done() {
                        let promise: js_sys::Promise = ev.value().into();
                        self.next_event = Some(SendWrapper::new(promise.into()));
                    }
                }
            }

            let event = if let Some(next_event) = self.next_event.as_mut() {
                let e = match Future::poll(Pin::new(&mut **next_event), cx) {
                    Poll::Ready(Ok(ev)) => ffi::ListenEvent::from(ev),
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
                };
                self.next_event = None;
                e
            } else {
                return Poll::Ready(None);
            };

            for addr in event
                .new_addrs()
                .into_iter()
                .flat_map(|e| e.to_vec().into_iter())
            {
                let addr = js_value_to_addr(&addr)?;
                self.pending_events
                    .push_back(ListenerEvent::NewAddress(addr));
            }

            for upgrade in event
                .new_connections()
                .into_iter()
                .flat_map(|e| e.to_vec().into_iter())
            {
                let upgrade: ffi::ConnectionEvent = upgrade.into();
                self.pending_events.push_back(ListenerEvent::Upgrade {
                    local_addr: upgrade.local_addr().parse()?,
                    remote_addr: upgrade.observed_addr().parse()?,
                    upgrade: futures::future::ok(Connection::new(upgrade.connection())),
                });
            }

            for addr in event
                .expired_addrs()
                .into_iter()
                .flat_map(|e| e.to_vec().into_iter())
            {
                match js_value_to_addr(&addr) {
                    Ok(addr) => self.pending_events.push_back(ListenerEvent::NewAddress(addr)),
                    Err(err) => self.pending_events.push_back(ListenerEvent::Error(err)),
                }
            }
        }
    }
}

/// Active stream of data with a remote.
///
/// It is guaranteed that each call to `io::Write::write` on this object maps to exactly one call
/// to `write` on the FFI. In other words, no internal buffering happens for writes, and data can't
/// be split.
pub struct Connection {
    /// The FFI object.
    inner: SendWrapper<ffi::Connection>,

    /// The iterator that was returned by `read()`.
    read_iterator: SendWrapper<js_sys::Iterator>,

    /// Reading part of the connection.
    read_state: ConnectionReadState,

    /// When we write data using the FFI, a promise is returned containing the moment when the
    /// underlying transport is ready to accept data again. This promise is stored here.
    /// If this is `Some`, we must wait until the contained promise is resolved to write again.
    previous_write_promise: Option<SendWrapper<JsFuture>>,
}

impl Connection {
    /// Initializes a `Connection` object from the FFI connection.
    fn new(inner: ffi::Connection) -> Self {
        let read_iterator = inner.read();

        Connection {
            inner: SendWrapper::new(inner),
            read_iterator: SendWrapper::new(read_iterator),
            read_state: ConnectionReadState::PendingData(Vec::new()),
            previous_write_promise: None,
        }
    }
}

/// Reading side of the connection.
enum ConnectionReadState {
    /// Some data have been read and are waiting to be transferred. Can be empty.
    PendingData(Vec<u8>),
    /// Waiting for a `Promise` containing the next data.
    Waiting(SendWrapper<JsFuture>),
    /// An error occurred or an earlier read yielded EOF.
    Finished,
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Connection").finish()
    }
}

impl AsyncRead for Connection {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize, io::Error>> {
        loop {
            match mem::replace(&mut self.read_state, ConnectionReadState::Finished) {
                ConnectionReadState::Finished => break Poll::Ready(Err(io::ErrorKind::BrokenPipe.into())),

                ConnectionReadState::PendingData(ref data) if data.is_empty() => {
                    let iter_next = self.read_iterator.next().map_err(JsErr::from)?;
                    if iter_next.done() {
                        self.read_state = ConnectionReadState::Finished;
                    } else {
                        let promise: js_sys::Promise = iter_next.value().into();
                        let promise = SendWrapper::new(promise.into());
                        self.read_state = ConnectionReadState::Waiting(promise);
                    }
                    continue;
                }

                ConnectionReadState::PendingData(mut data) => {
                    debug_assert!(!data.is_empty());
                    if buf.len() <= data.len() {
                        buf.copy_from_slice(&data[..buf.len()]);
                        self.read_state =
                            ConnectionReadState::PendingData(data.split_off(buf.len()));
                        break Poll::Ready(Ok(buf.len()));
                    } else {
                        let len = data.len();
                        buf[..len].copy_from_slice(&data);
                        self.read_state = ConnectionReadState::PendingData(Vec::new());
                        break Poll::Ready(Ok(len));
                    }
                }

                ConnectionReadState::Waiting(mut promise) => {
                    let data = match Future::poll(Pin::new(&mut *promise), cx) {
                        Poll::Ready(Ok(ref data)) if data.is_null() => break Poll::Ready(Ok(0)),
                        Poll::Ready(Ok(data)) => data,
                        Poll::Ready(Err(err)) => break Poll::Ready(Err(io::Error::from(JsErr::from(err)))),
                        Poll::Pending => {
                            self.read_state = ConnectionReadState::Waiting(promise);
                            break Poll::Pending;
                        }
                    };

                    // Try to directly copy the data into `buf` if it is large enough, otherwise
                    // transition to `PendingData` and loop again.
                    let data = js_sys::Uint8Array::new(&data);
                    let data_len = data.length() as usize;
                    if data_len <= buf.len() {
                        data.copy_to(&mut buf[..data_len]);
                        self.read_state = ConnectionReadState::PendingData(Vec::new());
                        break Poll::Ready(Ok(data_len));
                    } else {
                        let mut tmp_buf = vec![0; data_len];
                        data.copy_to(&mut tmp_buf[..]);
                        self.read_state = ConnectionReadState::PendingData(tmp_buf);
                        continue;
                    }
                }
            }
        }
    }
}

impl AsyncWrite for Connection {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        // Note: as explained in the doc-comments of `Connection`, each call to this function must
        // map to exactly one call to `self.inner.write()`.

        if let Some(mut promise) = self.previous_write_promise.take() {
            match Future::poll(Pin::new(&mut *promise), cx) {
                Poll::Ready(Ok(_)) => (),
                Poll::Ready(Err(err)) => return Poll::Ready(Err(io::Error::from(JsErr::from(err)))),
                Poll::Pending => {
                    self.previous_write_promise = Some(promise);
                    return Poll::Pending;
                }
            }
        }

        debug_assert!(self.previous_write_promise.is_none());
        self.previous_write_promise = Some(SendWrapper::new(
            self.inner.write(buf).map_err(JsErr::from)?.into(),
        ));
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        // There's no flushing mechanism. In the FFI we consider that writing implicitly flushes.
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        // Shutting down is considered instantaneous.
        match self.inner.shutdown() {
            Ok(()) => Poll::Ready(Ok(())),
            Err(err) => Poll::Ready(Err(io::Error::from(JsErr::from(err)))),
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.inner.close();
    }
}

/// Returns true if `err` is an error about an address not being supported.
fn is_not_supported_error(err: &JsValue) -> bool {
    if let Some(err) = err.dyn_ref::<js_sys::Error>() {
        if String::from(err.name()) == "NotSupportedError" {
            true
        } else {
            false
        }
    } else {
        false
    }
}

/// Turns a `JsValue` containing a `String` into a `Multiaddr`, if possible.
fn js_value_to_addr(addr: &JsValue) -> Result<Multiaddr, JsErr> {
    if let Some(addr) = addr.as_string() {
        Ok(addr.parse()?)
    } else {
        Err(JsValue::from_str("Element in new_addrs is not a string").into())
    }
}

/// Error that can be generated by the `ExtTransport`.
pub struct JsErr(SendWrapper<JsValue>);

impl From<JsValue> for JsErr {
    fn from(val: JsValue) -> JsErr {
        JsErr(SendWrapper::new(val))
    }
}

impl From<tetsy_libp2p_core::multiaddr::Error> for JsErr {
    fn from(err: tetsy_libp2p_core::multiaddr::Error) -> JsErr {
        JsValue::from_str(&err.to_string()).into()
    }
}

impl From<JsErr> for io::Error {
    fn from(err: JsErr) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err.to_string())
    }
}

impl fmt::Debug for JsErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for JsErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.0.as_string() {
            write!(f, "{}", s)
        } else if let Some(err) = self.0.dyn_ref::<js_sys::Error>() {
            write!(f, "{}", String::from(err.message()))
        } else if let Some(obj) = self.0.dyn_ref::<js_sys::Object>() {
            write!(f, "{}", String::from(obj.to_string()))
        } else {
            write!(f, "{:?}", &*self.0)
        }
    }
}

impl error::Error for JsErr {}
