use slog::*;

thread_local!( static TL_THREAD_ID: std::thread::ThreadId = std::thread::current().id() );


#[derive(Clone)]
//
pub struct ThreadLocalDrain<D> where D: Drain
{
	pub drain: D
}


impl<D> Drain for ThreadLocalDrain<D> where D: Drain
{
	type Ok  = ();
	type Err = ! ;

	fn log( &self, record: &Record, values: &OwnedKVList ) -> std::result::Result< Self::Ok, Self::Err >
	{
		let tid     = TL_THREAD_ID.with(|id| {*id} );

		let chained = OwnedKVList::from
		(
			OwnedKV( (SingleKV( "thread", format!( "{:?}", tid ) ), values.clone()) )
		);

		let _ = self.drain.log( record, &chained );

		Ok(())
	}
}


