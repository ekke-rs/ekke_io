use rand         :: { Rng                    };
use serde_derive :: { Serialize, Deserialize };

// u128 doesn't work in wasm and serde is being a pain, so 2 u64
//
/// Identifies a connection.
/// The id field is deliberately private, so we can change the actual implementation later.
/// Currently uses a 128bit random number.
//
#[ derive( Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize )]
//
pub struct ConnID
{
	a: u64,
	b: u64,
}


impl ConnID
{
	pub fn new() -> Self
	{
		let mut rng = rand::thread_rng();

		let a = rng.gen::<u64>();
		let b = rng.gen::<u64>();

		Self{ a, b }
	}
}
