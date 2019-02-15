use failure:: { Error, Fail  } ;
use actix  :: { MailboxError } ;


/// Custom result type, Allows to omit error type since it's always
/// [`failure::Error`](https://docs.rs/failure/0.1.5/failure/struct.Error.html).
///
pub type EkkeResult<T> = Result< T, Error >;


/// The specific errors ekke_io can return.
///
#[ derive( Debug, Fail ) ]
//
pub enum EkkeIoError
{
	#[ fail( display = "Cannot use socket before connecting" ) ]
	//
	UseSocketBeforeConnect,

	#[ fail( display = "Nobody connected to the socket" ) ]
	//
	NoConnectionsReceived,

	#[ fail( display = "Rpc: This is an error in ekke. The mailbox of the {} actor cannot keep up with the message flow, or it has been closed to early. If you run into this, please file an issue at https://github.com/najamelan/ekke. Actix Error: {}", _0, _1 ) ]
	//
	ActixMailboxError( String, MailboxError ),

	#[ fail( display = "Rpc: This is an error in ekke. The mailbox of the {} actor cannot keep up with the message flow, or it has been closed to early. If you run into this, please file an issue at https://github.com/najamelan/ekke", _0 ) ]
	//
	ActixSendError( String ),

	#[ fail( display = "No handler registered for service: {}", _0 ) ]
	//
	NoHandlerForService( String ),

	#[ fail( display = "Failed to downcast Any to Recepient<{}>. This is a bug in Ekke, please file an issue at https://github.com/najamelan/ekke.", _0 ) ]
	//
	DowncastRecipientFailed( String ),

	#[ fail( display = "Rpc: Handler for service [{}] is already registered. Second attempt was by: [{}].", _0, _1 ) ]
	//
	DoubleServiceRegistration( String, String ),
}
