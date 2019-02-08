use crate::{ IpcMessage, IpcConnTrack };
use std::rc::Rc;
use std::cell::RefCell;

use tokio::codec::Decoder;
use tokio_serde_cbor::{ Codec };

use actix::prelude::*;
use tokio_uds::UnixStream;

use tokio::codec::Framed;
use tokio::prelude::stream::{SplitSink, SplitStream};
use tokio::prelude::*;
use futures_util::{future::FutureExt, try_future::TryFutureExt};



/// Hides the underlying socket handling from client. The constructor takes a unix stream, but later will probably take any stream type. It also takes a Recipient<IpcConnTrack> to forward incoming messages to and it needs it's own address for setting up listening, so you should create this with `Actor::create` and `IpcPeer::new`.
/// Will forward any IpcMessage you send to it on the network stream serialized as cbor, and will send every incoming message to your dispatcher.
/// Currently uses Rc on a field because actix normally keeps an actor in the same thread. This might change later to make it Send and Sync.
///
#[ derive( Debug ) ]
//
pub struct IpcPeer
{
	sink: Rc<RefCell< SplitSink<Framed<UnixStream, Codec<IpcMessage, IpcMessage>>> >>
}

impl Actor for IpcPeer { type Context = Context<Self>; }


impl IpcPeer
{
	pub fn new( connection: UnixStream, dispatch: Recipient<IpcConnTrack>, addr: Addr<Self> ) -> Self
	{
		let codec: Codec<IpcMessage, IpcMessage>  = Codec::new().packed( true );

		let (sink, stream) = codec.framed( connection ).split();

		Arbiter::spawn( async move
		{
			await!( Self::listen( stream, dispatch, addr ) );

			Ok(())

		}.boxed().compat());

		Self
		{
			sink: Rc::new( RefCell::new( sink ))
		}

	}


	/// Will listen to a connection and send all incoming messages to the dispatch.
	///
	#[ inline ]
	//
	async fn listen( mut stream: SplitStream<Framed<UnixStream, Codec<IpcMessage, IpcMessage>>>, dispatch: Recipient<IpcConnTrack>, self_addr: Addr<Self> )
	{
		loop
		{
			let option: Option< Result< IpcMessage, _ > > = await!( stream.next() );

			let frame = match option
			{
				Some( result ) =>
				{
					match result
					{
						Ok ( frame ) => frame,
						Err( error ) =>
						{
							eprintln!( "Error extracting IpcMessage from stream: {:#?}", error );
							continue;
						}
					}
				},

				None => return     // Disconnected
			};


			Arbiter::spawn
			(
				dispatch.send( IpcConnTrack{ ipc_msg: frame, ipc_peer: self_addr.clone().recipient() } )

					.map(|_|())
					.map_err(|e| eprintln!( "IpcPeer::listen -> mailbox error: {}", e ))
			);
		}
	}
}



impl Handler< IpcMessage > for IpcPeer
{
	type Result = ();

	fn handle( &mut self, msg: IpcMessage, _ctx: &mut Context<Self> ) -> Self::Result
	{
		let sink = self.sink.clone();

		Arbiter::spawn( async move
		{
			let mut stay_alive = sink.borrow_mut();

			match await!( stay_alive.send_async( msg ) )
			{
				Ok (_) => { println! ( "Ekke: successfully wrote to stream"       ); },
				Err(e) => { eprintln!( "Ekke: failed to write to stream: {:?}", e ); }
			}

			Ok(())

		}.boxed().compat());
	}

}
