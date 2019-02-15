/// Macro for implementing actix::MessageResponse for a struct.
/// Taken from actix src/handler.rs
///
/// TODO: This would be nicer with a derive macro.
///
/// Example
///
/// ```
/// impl_message_response!( MyType );
/// ```
///
#[macro_export]
//
macro_rules! impl_message_response
{
	($type:ty) => {
		use actix::dev    :: { ResponseChannel, MessageResponse } ;
		impl<A, M> MessageResponse<A, M> for $type
		where
			A: Actor,
			M: Message<Result = $type>,
		{
			fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
				if let Some(tx) = tx {
					tx.send(self);
				}
			}
		}
	};
}
