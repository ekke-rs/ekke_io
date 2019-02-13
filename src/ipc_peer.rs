use std              :: { rc::Rc, cell::RefCell                       };

use actix::prelude   :: { *                                           };
use failure          :: { ResultExt                                   };
use futures_util     :: { future::FutureExt, try_future::TryFutureExt };
use slog             :: { Logger, error, info                         };

use tokio::prelude   :: { *, stream::{ SplitSink, SplitStream }       };
use tokio::codec     :: { Decoder, Framed                             };
use tokio_serde_cbor :: { Codec                                       };
use tokio_uds        :: { UnixStream                                  };

use crate            ::{ IpcMessage, IpcConnTrack, ResultExtSlog      };



/// Hides the underlying socket handling from client. The constructor takes a unix stream, but later will probably take any stream type. It also takes a Recipient<IpcConnTrack> to forward incoming messages to and it needs it's own address for setting up listening, so you should create this with `Actor::create` and `IpcPeer::new`.
/// Will forward any IpcMessage you send to it on the network stream serialized as cbor, and will send every incoming message to your dispatcher.
/// Currently uses Rc on a field because actix normally keeps an actor in the same thread. This might change later to make it Send and Sync.
///
#[ derive( Debug ) ]  #[allow(clippy::type_complexity)]
//
pub struct IpcPeer
{
	  sink: Rc<RefCell< SplitSink<Framed<UnixStream, Codec<IpcMessage, IpcMessage>>> >>
	, log : Logger
}

impl Actor for IpcPeer { type Context = Context<Self>; }


impl IpcPeer
{
	pub fn new( connection: UnixStream, dispatch: Recipient<IpcConnTrack>, addr: Addr<Self>, log: Logger ) -> Self
	{
		let codec: Codec<IpcMessage, IpcMessage>  = Codec::new().packed( true );

		let (sink, stream) = codec.framed( connection ).split();
		let listen_log     = log.clone();

		Arbiter::spawn( async move
		{
			await!( Self::listen( stream, dispatch, addr, listen_log ) );

			Ok(())

		}.boxed().compat());

		Self
		{
			  sink: Rc::new( RefCell::new( sink ))
			, log
		}

	}


	/// Will listen to a connection and send all incoming messages to the dispatch.
	///
	#[ inline ]
	//
	async fn listen( mut stream: SplitStream<Framed<UnixStream, Codec<IpcMessage, IpcMessage>>>, dispatch: Recipient<IpcConnTrack>, self_addr: Addr<Self>, log: Logger )
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
							error!( &log, "Error extracting IpcMessage from stream: {:#?}", error );
							continue;
						}
					}
				},

				None => return     // Disconnected
			};

			// TODO: don't clone every iteration!
			//
			let log_loop = log.clone();


			Arbiter::spawn
			(
				dispatch.send( IpcConnTrack{ ipc_msg: frame, ipc_peer: self_addr.clone().recipient() } )

					.then( move |r| { r.context( "IpcPeer::listen -> Dispatcher: mailbox error." ).unwraps( &log_loop ); Ok(())} )
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
