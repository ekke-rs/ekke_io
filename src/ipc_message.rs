//! Contains the message types for communicating with IpcPeer. You should send IpcMessage
//! to IpcPeer to send outgoing communication, except for specific scenarios like: Request/Response,
//! Ack, Broadcast, Publisher/Subscriber. See the respective Message types in this module
//! for more information.
//!
//! IpcPeer will send a ReceiveRequest to your Rpc for incoming requests. Usually the
//! dispatcher will hold on to the IpcPeer address and deserialize the IpcMessage payload
//! so your service can receive it's uniquely typed message.
//! Your service actor must return a response from it's handler for the request.
//

use actix::prelude:: { *                                } ;

use serde         :: { Serialize, Deserialize           } ;
use serde_cbor    :: { to_vec                           } ;

use crate         :: { impl_message_response, ConnID, EkkeIoError } ;


// No longer compiles if the doc is above this derive
//
impl_message_response!( IpcMessage );
//

/// Represents a message that goes over the wire. It always contains a string service name
/// to allow dispatching in the receiving application. A connection ID allows connection tracking.
///
///
#[ derive( Debug, Serialize, Deserialize, Message )]
//
pub struct IpcMessage
{
	/// The name of the service you are sending to.
	///
	pub service: String,

	/// Unique Connection Id. For the moment this is just a random 128bit number.
	/// When creating an initial request you can set this to track the response to the request.
	/// When responding, you should always send the same id that you got in the request. This
	/// is mainly used by Rpc internally, since when you send a request, you will have a future
	/// that resolves to the response.
	///
	pub conn_id: ConnID,


	/// Whether this message is a Request/Response/Ack/Broadcast/...
	///
	pub ms_type: MessageType,


	#[ serde( with = "serde_bytes") ]
	//
	/// cbor encoded Service Message
	///
	pub payload: Vec<u8>
}



impl IpcMessage
{
	/// Will serialize the payload, currently using cbor.
	///
	pub fn new
	(
		  service: String
		, payload: impl Serialize
		, ms_type: MessageType
		, conn_id: ConnID

	) -> Self
	{
		Self
		{
			  service
			, ms_type
			, conn_id
			, payload: to_vec( &payload ).unwrap()
		}
	}
}


/// When you want to call an rpc in another application, you should create a message of this type.
/// Note that this example uses unwraps on the MailboxError from actix. You shouldn't do that in
/// production code, but I have not yet found anything better in development than to wait until
/// I can make it happen with stress testing and figure a better solution from there.
///
///     let rpc = Rpc::new( log.new( o!( "Actor" => "Rpc" ) ), crate::service_map ).start();
///
///     let conn_id = ConnID::new();
///
///     let response = await!( rpc.send
///     (
///     	SendRequest
///     	{
///     		ipc_peer: ekke_server.recipient(),
///
///     		ipc_msg: IpcMessage::new
///     		(
///     			  "RegisterApplication".to_string()
///     			, RegisterApplication { conn_id, app_name: "Systemd".to_string() }
///     			, MessageType::SendRequest
///     			, conn_id
///     		)
///     	}
///
///     )).unwraps( &log );
///
#[ derive( Message ) ] #[ rtype( result="Result<IpcResponse, EkkeIoError>" ) ]
//
pub struct SendRequest{ pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }


/// This is a wrapper type around IpcMessage to allow implementing handlers for a specific message type.
/// Rpc will create this message type automatically to indicate the peer application that this
/// needs to be handled as a request. It also allows the Rpc Actor to implement a specific handler
/// for incoming requests. You shouldn't need to use this as a user of the framework.
///
#[ derive( Message ) ] pub struct ReceiveRequest { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }

/// This is a wrapper type around IpcMessage to allow implementing handlers for a specific message type.
/// Rpc will create this message type automatically to indicate the peer application that this
/// needs to be handled as a response to a request. It also allows the Rpc Actor to implement a specific handler
/// for incoming requests. You shouldn't need to use this as a user of the framework.
///
#[ derive( Message ) ] pub struct IpcResponse     { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }

/// This is a wrapper type around IpcMessage to allow implementing handlers for a specific message type.
/// Ipc message type indicating that an error happened before the message could be delivered to the actor handling
/// this message. Most notably this will happen when deserialization fails. Every application should always create
/// an actor that can handle these and that subscribes with the Rpc for this error type. Applications/Servers can create
/// other more specific error types that you should handle.
///
#[ derive( Message ) ] pub struct IpcError       { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }

/// This is a wrapper type around IpcMessage to allow implementing handlers for a specific message type.
/// Ipc message that indicates that the sender would like to be acknowleged of reception. This is a specific type of
/// request that you don't have to implement a service for. ekke_io will send Acknowledgements automatically.
/// This is mainly useful for working on unreliable transports.
/// This is currently not implemented and will most certainly change in the future, it should be ortagonal to any
/// message type, which this design doesn't allow.
///
#[ derive( Message ) ] pub struct PleaseAck      { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }
#[ derive( Message ) ] pub struct Ack            { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }

/// This is a wrapper type around IpcMessage to allow implementing handlers for a specific message type.
/// Create this message type if you want to broadcast to all connected peers. You should issue it with actix-broker
/// and all IpcPeer actors will subscribe to this.
///
#[ derive( Message ) ] pub struct Broadcast      { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }



/// Helps flow decisions for messages of type IpcMessage
///
#[ derive( Serialize, Deserialize, Debug ) ]
//
pub enum MessageType
{
	SendRequest   ,
	ReceiveRequest,
	Response      ,
	PleaseAck     ,
	Ack           ,
	Broadcast     ,
	Error         ,
}
