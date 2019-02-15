# ekke_io
Input/Output library for the ekke framework

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
  - clean up and document after request/response implementation
  - ipcpeer and ipcmessage: codec as option so people aren't obliged to use cbor
  - create modules that provide the streams to the client
  - implement ack/publish/subscribe/broadcast
  - Derive the service trait
  - try to make dependency on slog and typename optional
  - what does the double serialization cost us? Use Bytes for the outer one and CBOR for the inner one? or the other way around?
  - rpc: replace hashmap with hashbrown
  - only bind to one socket for all peer apps? peer authentication?


