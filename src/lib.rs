//! This is the io library for the ekke application framework. It has been written specifically for ekke, but can be used independantly from it. It is currently not considered production ready and not officially released.
//!
//! Currently it provides an infrastructure allowing Ipc communication with actix actors over unix domain sockets. It provides convenient actors for setting up and listening to the socket.
//!
//! It will send IpcRequestIn messages to an actor you provide that acts as a dispatcher. The ipc conntrack message contains the ipc peer address as a connection tracking means. Your dispatcher should deserialize the IpcMessage and send your service actor the service message. IpcMessage also contains a string name which allows your dispatcher to know to which actor to send the contained message.
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
	Ack            ,
	Broadcast      ,
	MessageType    ,
	IpcMessage     ,
	IpcError       ,
	IpcResponse    ,
	PleaseAck      ,
	IpcRequestIn ,
	IpcRequestOut    ,
};

pub use log::
{
	  ThreadLocalDrain
	, FnGuard
};


pub use rpc::
{
	  Rpc
	, register_service::RegisterService
	, register_service::RegisterServiceMethod
};


#[ cfg( feature = "http_server" ) ]
//
mod http_server;


#[ cfg( feature = "http_server" ) ]
//
pub use http_server::
{
	HttpServer     ,
	ResponseFuture ,
	Responder      ,
};



mod import
{
	#[ allow( unused_imports ) ]
	//
	pub( crate ) use
	{

		actix             :: { Actor, Addr, Arbiter, AsyncContext, Context, Handler, MailboxError,
			                        Message, Recipient, Supervised, SystemService, dev::ToEnvelope           },
		actix_async_await :: { ResponseStdFuture as ActixFuture                                             },

		failure           :: { Fail, Error, format_err, ResultExt as _                                      },

		futures           :: { channel, future::{ join_all, ok }                                            },
		futures_util      :: { future::{ FutureExt }, try_future::TryFutureExt                              },

		hashbrown         :: { HashMap                                                                      },
		rand              :: { Rng                                                                          },

		serde             :: { Serialize, Deserialize, de::DeserializeOwned                                 },
		serde_cbor        :: { from_slice as des                                                            },

		slog              :: { Drain, Logger, trace, debug, info, warn, error, crit, o                      },
		slog_unwraps      :: { ResultExt as _                                                               },

		std               :: { any::{ Any, TypeId }, cell::RefCell, convert::From, convert::TryFrom,
		                       env, fmt, future::Future as StdFuture, net::SocketAddr, path::PathBuf,
		                       process::Command, rc::Rc, sync::Arc, pin::Pin                                },

		// tokio::prelude::Future allows to use .then, but I imagine there is a better way...
		//
		tokio             :: { codec::{ Framed, Decoder }, io::{AsyncRead, AsyncWrite}, net::UnixStream,
		                       net::UnixListener                                                            },
		tokio::prelude    :: { Future as _, stream::{ SplitSink, SplitStream, Stream } },
		tokio_async_await :: { await as awaits, stream::StreamExt, sink::SinkExt                            },
		tokio_serde_cbor  :: { Codec                                                                        },

		typename          :: { TypeName                                                                     },
	};
}
