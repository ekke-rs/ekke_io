# ekke_io
Input/Output library for the ekke framework

NOT READY FOR USE

# Layers

## transport layer

- streams
- codecs

## communication layer

- announce (no response)
- request/response
- publish/subscribe
- broadcast

## User layer

- the actors the user implements
- dispatching to the right actor

### Announce

Send a message to a peer without response.

### Request/Response


# todo

- ipc:
  - either never put documentation at the module level, or make them public see rustdoc book for some attributes on where things appear
  - do we really need both a MessageType enum and wrapper types? yes we do but we could give the whole design another thought to make it more elegant.
  - ipcpeer and ipcmessage: codec as option so people aren't obliged to use cbor. Actually the deserialize methods in Rpc also are
    tied to cbor.
  - create modules that provide the streams to the client -> for now we shall put abstractions in the applications, because there is quite some ekke specific choices that might not be useful to other users... (choice to invoke apps with passing sock address over cli, the choice we will make to give an identifyer and use only one socket, ...)
  - implement ack/publish/subscribe/broadcast
  - try to make dependency on slog and typename optional
  - what does the double serialization cost us? Use Bytes for the outer one and CBOR for the inner one? or the other way around?
  - provide macro to generate the callback for rpc? or make it a trait object?
  - unit tests
  - fuzz/stress testing


