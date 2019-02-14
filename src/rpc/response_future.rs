use crate::ipc_message::Response;
use std::
{
	future  :: Future,
	pin     :: {Pin, /*Unpin*/},
	rc      :: Rc,
	cell    :: RefCell,
	task    :: {LocalWaker, Poll, Waker},
};




pub struct ResponseFuture
{
	waker   : Option< Waker    >,
	response: Rc<RefCell< Option< Response > >>,
}


impl Future for ResponseFuture
{
	type Output = Response;


	fn poll( mut self: Pin<&mut Self>, lw: &LocalWaker ) -> Poll<Self::Output>
	{
		if self.response.borrow().is_some()
		{
			Poll::Ready( self.response.replace( None ).unwrap() )
		}

		else
		{
			// Set waker so that the thread can wake up the current task
			// when the timer has completed, ensuring that the future is polled
			// again and sees that `completed = true`.
			//
			self.waker = Some( lw.clone().into_waker() );

			Poll::Pending
		}
	}
}



impl ResponseFuture
{
	/// Create a new `ResponseFuture` which will complete after the provided timeout.
	///
	pub fn new() -> Self
	{
		ResponseFuture
		{
			waker    : None ,
			response : Rc::new(RefCell::new( None )) ,
		}
	}



	pub fn response( &mut self, response: Response )
	{
		self.response.replace( Some( response ) );

		if let Some( waker ) = &self.waker
		{
			waker.wake();
		}
	}
}
