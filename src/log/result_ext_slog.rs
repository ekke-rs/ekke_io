use std::fmt::{ Debug, Display };
use backtrace::Backtrace;
use regex::Regex;
use slog::*;



pub trait ResultExtSlog<T>
{
	fn expects( self, log: Logger, msg: &str ) -> T;
	fn unwraps( self, log: Logger            ) -> T;
}


impl<T, E> ResultExtSlog<T> for std::result::Result<T, E> where E: Display + Debug
{
	fn expects( self, log: Logger, msg: &str ) -> T
	{
		self.map_err( |e|
		{
			error!( log, "{}: {} -> Error: {}" , demangle( "expects" ), msg, e );
			e

		}).unwrap()
	}


	fn unwraps( self, log: Logger ) -> T
	{
		self.map_err( |e|
		{
			error!( log, "{} -> Error: {}" , demangle( "unwraps" ), e );
			e

		}).unwrap()
	}
}


impl<T, E> ResultExtSlog<T> for std::result::Result<T, E> where E: Debug
{
	default fn expects( self, log: Logger, msg: &str ) -> T
	{
		self.map_err( |e|
		{
			error!( log, "{}: {} -> Error: {:?}" , demangle( "expects" ), msg, e );
			panic!();

		}).unwrap()
	}


	default fn unwraps( self, log: Logger ) -> T
	{
		self.map_err( |e|
		{
			error!( log, "{} -> Error: {:?}" , demangle( "unwraps" ), e );
			panic!();

		}).unwrap()
	}
}



// Demangle the API of the backtrace crate!
//
fn demangle( which: &str ) -> String
{
	let empty  = String::with_capacity(0);
	let bt     = Backtrace::new();
	let frames = bt.frames();

	let frame = &frames.get( 4 );

	if let Some( frame  ) = frame {
	if let Some( symbol ) = frame.symbols().last()
	{
		format!
		(
			  "PANIC - fn `{}` calls `{}` @ {}:{}"
			, symbol.name()    .map( |s| strip( format!( "{}", s ) )     ).unwrap_or_else(|| empty.clone())
			, which
			, symbol.filename().map( |s| s.to_string_lossy().to_string() ).unwrap_or_else(|| empty.clone())
			, symbol.lineno()  .map( |s| format!( "{}", s )              ).unwrap_or( empty )
		)

	} else { empty }
	} else { empty }
}



fn strip( input: String ) -> String
{
	let re = Regex::new( r"(\w+)::[[:alnum:]]+$" ).unwrap();

	re.captures( &input )

		.map( |caps|

			caps.get(1)

				.map_or( String::new(), |m| m.as_str().to_string() )

		)

		.unwrap_or( input )
}
