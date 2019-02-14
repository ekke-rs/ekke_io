//! Contains the message types for communicating with IpcPeer. You should send IpcMessage
//! to IpcPeer to send outgoing communication.
//!
//! IpcPeer will send a ReceiveRequest to your dispatcher for incoming requests. Usually the
//! dispatcher will hold on to the IpcPeer address and deserialize the IpcMessage payload
//! so your service can receive it's uniquely typed message.
//! Your service actor can send a response to this request by returning an IpcMessage from
//! the Handler::handle method. The dispatcher can then forward this to the correct IpcPeer.
//

use actix::prelude:: *                                    ;

use serde         :: Serialize                            ;
use serde_derive  :: { Serialize, Deserialize }           ;
use serde_cbor    :: to_vec                               ;

use crate         :: { impl_message_response, ConnID    } ;



/// An IpcPeer will send this to your dispatcher for incoming messages,
/// wrapped in ReceiveRequest. Holds the serialized actix message in payload.
///
/// You should use this to wrap outgoing ipc messages, and use the new method
/// so the payload gets serialized for you.
///
impl_message_response!( IpcMessage );
//
#[ derive( Debug, Serialize, Deserialize, Message )]
//
pub struct IpcMessage
{
	/// The name of the service you are sending to.
	///
	pub service: String,

	/// Unique Connection Id. For the moment this is just a random 128bit number.
	/// When creating an initial request you can set this to track the response to the request.
	/// When responding, you should always send the same id that you got in the request.
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


#[ derive( Message ) ] pub struct SendRequest    { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }
#[ derive( Message ) ] pub struct ReceiveRequest { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }
#[ derive( Message ) ] pub struct Response       { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }
#[ derive( Message ) ] pub struct Error          { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }
#[ derive( Message ) ] pub struct PleaseAck      { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }
#[ derive( Message ) ] pub struct Ack            { pub ipc_peer: Recipient< IpcMessage >, pub ipc_msg: IpcMessage }
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
