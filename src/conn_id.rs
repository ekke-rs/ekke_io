use rand::Rng;
use serde_derive  :: { Serialize, Deserialize }           ;

/// Identifies a connection.
/// The id field is deliberately private, so we can change the actual implementation later.
///
#[ derive( Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize )]
//
pub struct ConnID
{
	id: u128
}


impl ConnID
{
	pub fn new() -> Self
	{
		let mut rng = rand::thread_rng();

		Self{ id: rng.gen::<u128>() }
	}
}
