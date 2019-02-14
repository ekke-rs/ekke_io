use actix::{ prelude::*, dev::ToEnvelope };
use typename::TypeName;
use crate::{ Rpc, RegisterService };
use std::any::TypeId;

pub trait Service

	where
	Self: Actor + TypeName,
	<Self as Actor>::Context: AsyncContext<Self>
{

	fn register_service<M>( &self, dispatcher: &Addr< Rpc >, ctx: &mut Self::Context )

	where

		Self: Handler<M>,
		<Self as Actor>::Context: ToEnvelope<Self, M>,
		M: Message + TypeName + Send + Message<Result = ()> + 'static
	{
		dispatcher.do_send
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

impl<A> Service for A
where
	Self: Actor + TypeName,
	<Self as Actor>::Context: AsyncContext<Self>
{}
