use crate :: { import::*      };
use crate ::{ Rpc, IpcMessage };



/// The message type for registering your services with the Rpc component. After providing
/// the callback function to Rpc which allows it to know the types to deserialize incoming
/// messages to, and registering, the rpc component will automatically forward incoming messages
/// to your actor.
///
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



/// This trait creates a convenient way for your Service Actors to register themselves
/// with the Rpc component. The registering is needed so that the Rpc component would
/// know to which actor to send an incoming message. The choice is based on the type of
/// the message. Hence there can only be one actor for each service, but an actor can
/// provide multiple services.
///
/// Currently this requires that your actor and your message types
/// derive [`Typename`](https://docs.rs/typename/0.1.0/typename/trait.TypeName.html)
/// from the typename crate. This is to allow [`Rpc`](struct.Rpc.html) to give you sensible
/// error messages. This restriction might be lifted in the future, but know that typename
/// is a very light dependency and probably worth the better error messages.
///
/// A service in ekke is an actor that can receive a request and that promises to return
/// a response. It is the typical remote procedure call.
///
/// When you import this trait, the actors in the file automatically get a new method on
/// self. See the example for usage:
///
///     use ekke_io::RegisterServiceMethod;
///
///     #[ derive( Debug, Clone, TypeName ) ]
///     //
///     pub struct Ekke
///     {
///     	pub log: Logger
///     }
///
///     impl Actor for Ekke
///     {
///     	type Context = Context<Self>;
///
///     	// Start the server
///     	// Register our services with the rpcer
///     	//
///     	fn started( &mut self, ctx: &mut Self::Context )
///     	{
///     		// Create the rpc actor. We are using slog for structured logging here.
///     		//
///     		let rpc = Rpc::new( self.log.new( o!( "Actor" => "Rpc" ) ), crate::service_map ).start();
///
///     		// Tell rpc that we provide the service for requests of type RegisterApplication
///     		// This method would be even more useful if you wouldn't have to pass a reference to rpc
///     		// but if rpc would become an actix service who's address can be found in the registry,
///     		// that would make it harder for users to replace the rpc component if wanted.
///     		//
///     		// The main reason for this trait is to reduce boilerplate in your actors.
///     		//
///     		self.register_service::<RegisterApplication>( &rpc, ctx );
///
///     		//...
///     	}
///   }
///
pub trait RegisterServiceMethod

	where
	Self: Actor + TypeName,
	<Self as Actor>::Context: AsyncContext<Self>
{

	/// See trait documentation for docs
	///
	fn register_service<M>( &self, rpc: &Addr< Rpc >, ctx: &mut Self::Context )

	where

		  Self                     : Handler<M, Result = IpcMessage>
		, M                        : Message<   Result = IpcMessage> + Message + TypeName + Send + 'static
		, <Self as Actor>::Context : ToEnvelope<Self, M>
	{
		// We use do_send, because it doesn't need to be async
		//
		rpc.do_send
		(
			RegisterService
			{
				service  : M::type_name(),
				actor    : Self::type_name(),
				type_id  : TypeId::of::<M>(),
				recipient: ctx.address().recipient::<M>()
			}
		)
	}
}



/// Implemented for all actors in your file if you import the trait
///
impl<A> RegisterServiceMethod for A

where
	    Self                   : Actor + TypeName
	 , <Self as Actor>::Context: AsyncContext<Self>

{}
