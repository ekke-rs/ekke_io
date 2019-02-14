use std::any::{ Any, TypeId };
use std               :: { collections::HashMap                           } ;

use actix             :: { prelude::*                                     } ;
use failure           :: { ResultExt                                      } ;
use serde_cbor        :: { from_slice as des                              } ;
use serde             :: { de::DeserializeOwned                              } ;

use slog              :: { Logger                                         } ;
use tokio::prelude    :: { Future                                         } ;
use tokio_async_await :: { await                                          } ;

use futures_util     :: { future::FutureExt, try_future::TryFutureExt };


use crate             :: { EkkeIoError, MessageType, ConnID } ;
use crate           :: { ReceiveRequest, IpcMessage, ResultExtSlog,       } ;



#[ derive( Debug ) ]
//
pub struct Rpc
{
	  handlers: HashMap< TypeId, Box< dyn Any > >
	, log     : Logger
	, matcher : fn( &Self, IpcMessage, Recipient< IpcMessage > )
}

impl Actor for Rpc { type Context = Context<Self>; }



impl Rpc
{
	pub fn new( log: Logger, matcher: fn( &Self, IpcMessage, Recipient< IpcMessage > ) ) -> Self
	{
		Self { handlers: HashMap::new(), log, matcher }
	}


	/// Send an error message back to the peer application over the ipc channel.
	///
	fn error_response( &self, error: String, addr: Recipient< IpcMessage > )
	{
		let log = self.log.clone();

		Arbiter::spawn
		(

			addr.send( IpcMessage::new( "EkkeServerError".into(), error, MessageType::Error, ConnID::new() ) )

				.then( move |r| { r.context( "Rpc::error_response -> IpcPeer: mailbox error." ).unwraps( &log ); Ok(())} )

		);
	}



	pub fn deserialize<INTO>( &self, msg: IpcMessage, ipc_peer: Recipient< IpcMessage > )

		where

			INTO: DeserializeOwned + Message + Send + 'static,
			INTO: Message< Result = IpcMessage >,
			INTO::Result: Send,
	{
		let name = msg.service.clone();
		let log  = self.log.clone();

		// Get the service handler out of our hashmap
		//
		match self.handlers.get( &TypeId::of::<INTO>() )
		{
			// If we have a handler for this service
			//
			Some( service ) =>
			{
				// Deserialize the payload
				//
				let de: INTO = match des( &msg.payload )
				{
					Ok ( data  ) => data,

					Err( error ) =>
					{
						// If we can't deserialize, send an error message to the ipc peer application
						//
						self.error_response
						(
							  format!( "Ekke Server could not deserialize your cbor data for service:{} :{:?}", &msg.service, error )
							, ipc_peer
						);

						// If we can't deserialize the message, there's no point in continuing to handle this request.
						//
						return;
					}
				};


				// Downcast our Any pointer
				//
				let recipient = match service.downcast_ref::< Recipient<INTO> >()
				{
					Some( recipient ) => recipient,
					None              => Err( EkkeIoError::DowncastRecipientFailed( name.clone() ) ).unwraps( &self.log )
				};


				// Send the message to the service actor and wait for a response to send back to the peer
				//
				let addr = recipient.clone();

				Arbiter::spawn( async move
				{
					let try_resp = await!( addr.send( de ) );

					let resp = try_resp.context( format!( "Rpc::Handler<ReceiveRequest> -> {}: mailbox error.", &name ) )
					    .unwraps( &log );

					await!( ipc_peer.send( resp ) ).unwraps( &log );


					Ok(())

				}.boxed().compat() )
			},

			// There is no handler for this service, let the peer app know
			// Note that this can also happen if there is a service but it's actor hasn't registered yet
			// We no longer keep a list of all services, only of registered actors.
			//
			None => self.error_response

				( format!( "Ekke Server received request for unknown service: {:?}", &msg.service ), ipc_peer )
		}
	}
}



/// Handle incoming IPC messages
///
impl Handler<ReceiveRequest> for Rpc
{
	type Result = ();


	/// Handle incoming IPC messages
	///
	///
	fn handle( &mut self, msg: ReceiveRequest, _ctx: &mut Context<Self> ) -> Self::Result
	{
		// Give user supplied callback the the data, so they can identify the type
		//
		(self.matcher)( self, msg.ipc_msg, msg.ipc_peer );
	}
}


/// We need to keep a list of service->actor handler mappings at runtime. This is where services
/// register.
///
impl<M> Handler<RegisterService<M>> for Rpc

where

	M: Send + Message<Result = IpcMessage> + 'static
{
	type Result = ();


	#[ allow( clippy::suspicious_else_formatting ) ]
	//
	fn handle( &mut self, msg: RegisterService<M>, _ctx: &mut Context<Self> ) -> Self::Result
	{
		if let Some( _service ) = self.handlers.remove( &msg.type_id )
		{
			let _:() =

				Err( EkkeIoError::DoubleServiceRegistration( format!( "{:?}", &msg.service ), msg.actor ) )

					.unwraps( &self.log );
		}

		else
		{
			self.handlers.insert( msg.type_id, Box::new( msg.recipient ) );
		}
	}
}



#[ derive( Message ) ]
//
pub struct RegisterService<M>

where
	M: Message<Result = IpcMessage> + Send + 'static
{
	pub service  : String,
	pub actor    : String,
	pub type_id  : TypeId,
	pub recipient: Recipient<M>
}

