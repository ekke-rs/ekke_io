use crate :: { import::*, Rpc };

use hyper:: { Body, Request, Response, Server, service::service_fn };


pub type ResponseFuture = Pin< Box< dyn StdFuture< Output = Result< Response<Body>, hyper::Error > > + Send > >;
pub type Responder      = Box< Fn( Request<Body>, Addr<Rpc>, Logger ) -> ResponseFuture + Send + Sync + 'static >;


#[ derive( Clone ) ]
//
pub struct HttpServer
{
	log    : Logger          ,
	handler: Arc< Responder >,
	rpc    : Addr<Rpc>       ,
}


impl HttpServer
{
	pub fn new( log: Logger, handler: Responder, rpc: Addr<Rpc> ) -> Self
	{
		Self
		{
			log,
			rpc,
			handler: Arc::new( handler ),
		}
	}


	pub async fn run( &self, addr: SocketAddr )
	{
		info!( self.log, "Listening on http://{}", addr );

		// Create a server bound on the provided address
		//
		let serve_future = Server::bind( &addr )

			// Serve requests using our `async serve_req` function.
			// `serve` takes a closure which returns a type implementing the
			// `Service` trait. `service_fn` returns a value implementing the
			// `Service` trait, and accepts a closure which goes from request
			// to a future of the response. In order to use our `serve_req`
			// function with Hyper, we have to box it and put it in a compatability
			// wrapper to go from a futures 0.3 future (the kind returned by
			// `async fn`) to a futures 0.1 future (the kind used by Hyper).
			//
			.serve( ||
			{
				let cb  = self.handler.clone()                             ;
				let rpc = self.rpc    .clone()                             ;
				let log = self.log    .new( o!( "fn" => "http_closure" ) ) ;

				service_fn( move |req| cb( req, rpc.clone(), log.clone() ).compat() )

			});

		// Wait for the server to complete serving or exit with an error.
		// If an error occurred, print it to stderr.
		//
		if let Err(e) = awaits!( serve_future )
		{
			error!( self.log, "server error: {}", e );
		}
	}
}
