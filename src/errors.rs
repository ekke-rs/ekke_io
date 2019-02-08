use failure::{ Error, Fail };


pub type EkkeResult<T> = Result< T, Error >;


#[ derive( Debug, Fail ) ]
//
pub enum EkkeError
{
	#[ fail( display = "Cannot use socket before connecting" ) ]
	//
	UseSocketBeforeConnect,

	#[ fail( display = "Nobody connected to the socket" ) ]
	//
	NoConnectionsReceived
}

