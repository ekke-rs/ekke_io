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
Give a com interface an IpcPeer, an IpcMessage? and receive a future that will resolve to the response.

What is the response type?
What does it all mean for apps in other languages?

# todo

- ipc:
  - dispatcher: maybe take new message types instead of IpcConntrack? Let IpcMessage have a `reply_to` field with an Option< Recipient<IpcMessage> > and a conn_id
  - try to make dependency on slog and typename optional
  - ipcpeer and ipcmessage: codec as option so people aren't obliged to use cbor
  - what does the double serialization cost us? Use Bytes for the outer one and CBOR for the inner one? or the other way around?
  - dispatcher: replace hashmap with hashbrown
  - Connection tracking in
  - only bind to one socket for all peer apps? peer authentication?
  - Recipient does not implement debug
  - maybe let dispatcher take a trait object
  - create modules that provide the streams to the client


# Event based dispatching

Eventloop service actor

```rust
Response  < RegisterApplication >
Request   < RegisterApplication >
Broadcast < RegisterApplication >
Announce  < RegisterApplication >
Ack       < RegisterApplication >

"RegisterApplication"


```

## Request response flow

Actor sends Request to Rpc
Rpc takes Request -> future to RegisterApplicationResponse
Rpc transforms Request to IpcMessage
Rpc sends to peer
---
IpcPeer receives IpcMessage
send IpcMessage to Rpc
Rpc sends RegisterApplication to Actor
Actor makes RegisterApplicationResponse containing same conn_id
Actor sends IpcMessage to peer
---
Ipcpeer receives Response as IpcMessage
Ipcpeer sends IpcMessage to Rpc
Rpc matches conn_id to hashtable
Rpc wakes up future

# todo
- rename dispatcher to Rpc
- Create request/response/ack/...types
