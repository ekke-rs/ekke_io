use std::
{
	  any::Any
	, any::TypeId
	, rc::Rc
	, cell::RefCell
};

use actix             :: { prelude::*                                     } ;
use actix_async_await :: { ResponseStdFuture as ActixFuture               } ;
use failure           :: { ResultExt as _                                 } ;
use hashbrown         :: { HashMap                                        } ;
use serde_cbor        :: { from_slice as des                              } ;
use serde             :: { de::DeserializeOwned                           } ;

use slog              :: { Logger, crit, debug, o                         } ;
use slog_unwraps      :: { ResultExt as _                                 } ;

use tokio::prelude    :: { Future                                         } ;
use tokio_async_await :: { await                                          } ;

use futures_util      :: { future::FutureExt, try_future::TryFutureExt    } ;
use futures           :: { channel::oneshot                               } ;

use crate::
{
	  EkkeIoError     ,
	  EkkeResult      ,
	  MessageType     ,
	  ConnID          ,
	  ReceiveRequest  ,
	  SendRequest     ,
	  IpcResponse     ,
	  IpcError        ,
	  IpcMessage      ,
	  RegisterService ,
};


pub(crate) mod register_service;


/// Rpc acts as an intermediary between your actors and IpcPeer. By registering your services with rpc, it will
/// make sure that message of that type arrive at your actor. See RegisterService. It also takes care of matching
/// a request to a response. When you send a SendRequest message to Rpc, you will get back a future that will
/// resolve to the reponse from a remote application.
///
pub struct Rpc
{
	handlers : HashMap< TypeId, Box< dyn Any > >                             ,
	responses: Rc<RefCell< HashMap< ConnID, oneshot::Sender< Result<IpcResponse, EkkeIoError> > > >> ,
	log      : Logger                                                        ,
	matcher  : fn( &Self, Logger, IpcMessage, Recipient< IpcMessage > )              ,
}

impl Actor for Rpc { type Context = Context<Self>; }



impl Rpc
{
	/// Create a new Rpc component.
	///
	/// The matcher parameter is a method that should take an IpcMessage, use the String ipc_msg.service
	/// to Identify the type of the object and which should call back rpc to continue processing the
	/// incoming message. The reason for this construction is that deserialize needs a static type to
	/// deserialize into, but we only know the type at runtime. We want Rpc to be reusable accross several
	/// applictions, which each will have their own list of services. I haven't found a more elegant way
	/// to achieve this than with this callback.
	///
	///     use crate::services::*;
	///     use ekke_io::{ IpcMessage, Rpc };
	///     use actix::Recipient;
	///
	///     pub( crate ) fn service_map( rpc: &Rpc, msg: IpcMessage, ipc_peer: Recipient< IpcMessage > )
	///     {
	///         match msg.service.as_ref()
	///         {
	///             "RegisterApplication" => rpc.deser_into::<RegisterApplication>( msg, ipc_peer ),
	///             _ =>(),
	///         }
	///     }
	///
	///     // when constructing an Rpc component, pass the callback as:
	///     //
	///     let rpc = Rpc::new( log.new( o!( "Actor" => "Rpc" ) ), crate::service_map ).start();
	///
	pub fn new( log: Logger, matcher: fn( &Self, Logger, IpcMessage, Recipient< IpcMessage > ) ) -> Self
	{
		Self { handlers: HashMap::new(), responses: Rc::new( RefCell::new( HashMap::new() )), log, matcher }
	}


	/// Send an error message back to the peer application over the ipc channel.
	///
	pub fn error_response( &self, service: String, error: String, addr: Recipient< IpcMessage >, conn_id: ConnID )
	{
		let log = self.log.clone();

		Arbiter::spawn
		(

			addr.send( IpcMessage::new( service, error, MessageType::Error, conn_id ) )

				.then( move |r| { r.context( "Rpc::error_response -> IpcPeer: mailbox error." ).unwraps( &log ); Ok(())} )

		);
	}


	/// Deserialize a message if you know the resulting type. This does not handle the error if deserialization
	/// fails, but rather returns you a Result.
	///
	/// TODO: Add Example
	///
	pub fn deserialize<INTO>( payload: Vec<u8> ) -> EkkeResult< INTO >

	where INTO: DeserializeOwned

	{
		let de: INTO = match des( &payload )
		{
			Ok ( data  ) => data,

			Err( error ) =>
			{
				return Err( error.into() );
			}
		};

		Ok( de )
	}



	/// Part of processing incoming requests, this method needs to be called from the callback function
	/// you pass to the constructor of this class. Look at the documentation on `new` for an example.
	///
	pub fn deser_into<INTO>( &self, msg: IpcMessage, ipc_peer: Recipient< IpcMessage > )

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
							  msg.service.clone()
							, format!( "Rpc component could not deserialize your message for service:{} :{:?}", &msg.service, error )
							, ipc_peer
							, msg.conn_id
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

				( msg.service.clone(), format!( "No handler is registered for service: {:?}", &msg.service ), ipc_peer, msg.conn_id )
		}
	}
}



/// Handle incoming IPC requests
///
impl Handler<ReceiveRequest> for Rpc
{
	type Result = ();


	/// Handle incoming IPC requests
	///
	fn handle( &mut self, msg: ReceiveRequest, _ctx: &mut Context<Self> ) -> Self::Result
	{
		debug!( &self.log, "Received incoming request: {}", &msg.ipc_msg.service );

		// Give user supplied callback the the data, so they can identify the type for deserialization
		//
		(self.matcher)( self, self.log.new( o!( "fn" => "service_map" ) ), msg.ipc_msg, msg.ipc_peer );
	}
}



/// Handle outgoing RPC requests
///
impl Handler<SendRequest> for Rpc
{
	type Result = ActixFuture< Result<IpcResponse, EkkeIoError> >;


	/// Handle outgoing RPC requests
	///
	fn handle( &mut self, mut msg: SendRequest, _ctx: &mut Context<Self> ) -> Self::Result
	{
		let (sender, receiver) = oneshot::channel::< Result<IpcResponse, EkkeIoError> >();

		self.responses.borrow_mut().insert( msg.ipc_msg.conn_id, sender );

		msg.ipc_msg.ms_type = MessageType::ReceiveRequest;
		let _ = msg.ipc_peer.do_send( msg.ipc_msg );


		let log = self.log.clone();

		ActixFuture::from( async move
		{
			await!( receiver ).unwraps( &log )
		})
	}
}



/// Handle incoming Responses
///
impl Handler<IpcResponse> for Rpc
{
	type Result = ();

	/// Handle incoming Responses
	///
	fn handle( &mut self, msg: IpcResponse, _ctx: &mut Context<Self> ) -> Self::Result
	{
		let mut borrow  = self.responses.borrow_mut();
		let     channel = borrow.remove( &msg.ipc_msg.conn_id ).unwrap();

		let     _       = channel.send( Ok( msg ) );
	}
}



/// Handle incoming Errors
///
impl Handler<IpcError> for Rpc
{
	type Result = ();

	/// Handle incoming Errors
	///
	fn handle( &mut self, msg: IpcError, _ctx: &mut Context<Self> ) -> Self::Result
	{
		let mut borrow  = self.responses.borrow_mut();
		let     channel = borrow.remove( &msg.ipc_msg.conn_id ).unwrap();

		let     _       = channel.send( Err( msg.into() ) );
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
			crit!( self.log, "{}", EkkeIoError::DoubleServiceRegistration( format!( "{:?}", &msg.service ), msg.actor ) );

			std::process::exit( 1 );
		}

		else
		{
			self.handlers.insert( msg.type_id, Box::new( msg.recipient ) );
		}
	}
}



