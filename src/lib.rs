//! This is the io library for the ekke application framework. It has been written specifically for ekke, but can be used independantly from it. It is currently not considered production ready and not officially released.
//!
//! Currently it provides an infrastructure allowing Ipc communication with actix actors over unix domain sockets. It provides convenient actors for setting up and listening to the socket.
//!
//! It will send IpcConnTrack messages to an actor you provide that acts as a dispatcher. The ipc conntrack message contains the ipc peer address as a connection tracking means. Your dispatcher should deserialize the IpcMessage and send your service actor the service message. IpcMessage also contains a string name which allows your dispatcher to know to which actor to send the contained message.
//!
//! The ekke_io crate provides all Io functionality for the ekke server as well as being a library that can be compiled to a shared object library for applications to link against. It can thus be used for ffi to allow building applications in other languages that commmunicate with Ekke.
//!
//! Contains:
//!
//! - Ipc functionality (currently over unix domain sockets, but should become cross platform)
//!   It could abstract out over all possible mechanisms, as stdin/stdout, ...
//! - Http Server for frontends (websockets)
//
#![ forbid( unsafe_code ) ]
#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures ) ]

mod actix_helpers;
mod conn_id;
mod rpc;
mod errors;
mod ipc_peer;
mod ipc_message;
mod log;
mod service;

pub use conn_id::
{
	  ConnID
};


pub use errors::
{
	  EkkeResult
	, EkkeIoError
};


pub use ipc_peer::
{
	  IpcPeer
};


pub use ipc_message::
{
	  IpcMessage
	, IpcConnTrack
	, SendRequest
	, ReceiveRequest
	, Response
	, Error
	, PleaseAck
	, Ack
	, Broadcast
	, MessageType
};

pub use log::
{
	  ThreadLocalDrain
	, FnGuard
	, ResultExtSlog
};


pub use rpc::
{
	  Rpc
	, RegisterService
};


pub use service::
{
	  Service
};

