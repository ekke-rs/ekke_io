use std::fmt::Display;
use std::fmt::{ Debug };
use backtrace::Backtrace;
use regex::Regex;
use slog::{ Logger, trace, debug, info, warn, error, crit, Level };
// use failure::{ Fail };


/// Add extras to the result type to ease logging of errors.
///
pub trait ResultExtSlog<T, E>

	where E: Display + Debug
{
	/// Logs the error to the provided logger before unwrapping.
	///
	fn unwraps( self, log: &Logger                   ) -> T;

	/// logs a potential error in the result and returns the result intact.
	///
	fn log    ( self, log: &Logger, lvl: slog::Level ) -> Result<T,E>;
}


impl<T, E> ResultExtSlog<T, E> for Result<T, E> where E: Display + Debug
{
	fn unwraps( self, log: &Logger ) -> T
	{
		self.map_err( |e|
		{
			crit!( log, "{} -> Error: {}" , demangle( "unwraps" ), e );
			e

		}).unwrap()
	}


	fn log( self, log: &Logger, lvl: Level ) -> Result<T, E>
	{
		self.map_err( |e|
		{
			match lvl
			{
				Level::Trace    => trace!( log, "{}", e ),
				Level::Debug    => debug!( log, "{}", e ),
				Level::Info     => info! ( log, "{}", e ),
				Level::Warning  => warn! ( log, "{}", e ),
				Level::Error    => error!( log, "{}", e ),
				Level::Critical => crit! ( log, "{}", e ),
			}

			e
		})
	}
}



// Demangle the API of the backtrace crate!
//
// Returns the caller function name + file:lineno for logging in ResultExtSlog
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



// Will return the function name from a string returned by backtrace:
//
// ekke::main::dkk39ru458u3 -> main
//
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

