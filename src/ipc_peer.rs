use std               :: { rc::Rc, cell::RefCell                       } ;

use actix::prelude    :: { *                                           } ;
use futures_util      :: { future::FutureExt, try_future::TryFutureExt } ;
use slog              :: { Logger, error, info                         } ;

use tokio::prelude    :: { *, stream::{ SplitSink, SplitStream }       } ;
use tokio::codec      :: { Decoder, Framed                             } ;
use tokio_serde_cbor  :: { Codec                                       } ;
use tokio_async_await :: { await                                       } ;

use crate::{ IpcMessage, ReceiveRequest, Response, ResultExtSlog, MessageType, Rpc };



/// Hides the underlying socket handling from client. The constructor takes a unix stream,
/// but later will probably take any stream type. It also takes a Recipient<ReceiveRequest>
/// to forward incoming messages to and it needs it's own address for setting up listening,
/// so you should create this with `Actor::create` and `IpcPeer::new`.
/// Will forward any IpcMessage you send to it on the network stream serialized as cbor,
/// and will send every incoming message to your rpc.
///
/// Currently uses Rc on a field because actix normally keeps an actor in the same thread.
/// This might change later to make it Send and Sync.
///
#[ derive( Debug ) ]  #[allow(clippy::type_complexity)]
//
pub struct IpcPeer<S>

	where S: AsyncRead + AsyncWrite

{
	  sink: Rc<RefCell< SplitSink<Framed<S, Codec<IpcMessage, IpcMessage>>> >>
	, log : Logger
}

impl<S> Actor for IpcPeer<S> where S: AsyncRead + AsyncWrite + 'static
{ type Context = Context<Self>; }


impl<S> IpcPeer<S>

	where S: AsyncRead + AsyncWrite + 'static

{
	pub fn new( connection: S, rpc: Addr<Rpc>, addr: Addr<Self>, log: Logger ) -> Self
	{
		let codec: Codec<IpcMessage, IpcMessage>  = Codec::new().packed( true );

		let (sink, stream) = codec.framed( connection ).split();
		let listen_log     = log.clone();

		Arbiter::spawn( async move
		{
			await!( Self::listen( stream, rpc, addr, listen_log ) );

			Ok(())

		}.boxed().compat());

		Self
		{
			  sink: Rc::new( RefCell::new( sink ))
			, log
		}

	}


	/// Will listen to a connection and send all incoming messages to the rpc.
	///
	#[ inline ]
	//
	async fn listen( mut stream: SplitStream<Framed<S, Codec<IpcMessage, IpcMessage>>>, rpc: Addr<Rpc>, self_addr: Addr<Self>, log: Logger )
	{
		loop
		{
			let option: Option< Result< IpcMessage, _ > > = await!( stream.next() );

			let frame = match option
			{
				Some( connection ) =>
				{
					match connection
					{
						Ok ( frame ) => frame,
						Err( error ) =>
						{
							error!( &log, "Error extracting IpcMessage from stream: {:#?}", error );
							continue;
						}
					}
				},

				None => return     // Disconnected
			};

			// Wrap ipc message, so that the correct handler can be called in Rpc
			// We spawn the future immediately here to avoid blocking the loop which should start processing
			// the next message.
			//
			let log_loop = log.clone();
			let rpc      = rpc.clone();
			let peer     = self_addr.clone().recipient();

			Arbiter::spawn( async move	{ match frame.ms_type
			{
				MessageType::ReceiveRequest =>

					await!( rpc.send( ReceiveRequest{ ipc_msg: frame, ipc_peer: peer } ) ).unwraps( &log_loop ),

				MessageType::Response =>

					await!( rpc.send( Response      { ipc_msg: frame, ipc_peer: peer } ) ).unwraps( &log_loop ),

				_ =>
				{
					error!( log_loop, "Unimplemented message type: {:?}", frame.ms_type );
					unreachable!()
				}

			};	Ok(()) }.boxed().compat());
		}
	}
}



impl<S> Handler< IpcMessage > for IpcPeer<S>

	where S: AsyncRead + AsyncWrite + 'static

{
	type Result = ();

	fn handle( &mut self, msg: IpcMessage, _ctx: &mut Context<Self> ) -> Self::Result
	{
		let sink = self.sink.clone();
		let log  = self.log .clone();

		Arbiter::spawn( async move
		{
			let mut stay_alive = sink.borrow_mut();

			match await!( stay_alive.send_async( msg ) )
			{
				Ok (_) => { info! ( log, "Ekke: successfully wrote to stream"       ); },
				Err(e) => { error!( log, "Ekke: failed to write to stream: {:?}", e ); }
			}

			Ok(())

		}.boxed().compat());
	}

}
