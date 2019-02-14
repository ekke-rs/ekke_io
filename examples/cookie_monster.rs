use actix::prelude::*;
use ekke_io::*;
use slog::*;
use serde::{Serialize, Deserialize};




#[ derive( Service ) ]
//
struct CookieMonster {}


impl Actor for CookieMonster
{
	type Context = Context<Self>;

	fn started( &mut self, ctx: &mut Self::Context )
	{
		let dispatcher = Dispatcher::new( log.new( o!( "Actor" => "CookieMonster" ) ), service_map ).start();

		self.register_service::<RegisterApplication>( &dispatcher, ctx );
	}
}


impl Handler<Cookie> for CookieMonster
{
	type Result = ();

	fn handle( &mut self, msg: Cookie, _ctx: &mut Context<Self> ) -> Self::Result
	{
		println!( "Ekke: Received app registration for app: {}", msg.app_name );
	}

}






/// This function maps the
pub(crate) fn service_map( msg: IpcMessage, d: &Dispatcher )
{
    match msg.ipc_msg.service.as_ref()
    {
        "Cookie" => d.deserialize::<Cookie>( msg ),
        _ =>(),
    }
}
