use slog::*;

pub struct FnGuard
{
	function_name: &'static str,
	logger       : Logger      ,
}


impl FnGuard
{
	pub fn new<T>( in_logger: Logger, values: OwnedKV<T>, function_name: &'static str ) -> FnGuard

		where T: SendSyncRefUnwindSafeKV + 'static

	{
		let logger = in_logger.new( values );

		info!( logger, "[Enter]"; o!( "function_name" => function_name ) );

		FnGuard { function_name, logger }
	}



	pub fn sub_guard(&self, function_name: &'static str) -> FnGuard
	{
		FnGuard::new( self.logger.clone(), o!(), function_name )
	}



	pub fn log( &self, record: &Record ) { self.logger.log( record ) }
}


impl Drop for FnGuard
{
	fn drop(&mut self)
	{
		info!( self.logger, "[Exit]"; o!("function_name"=>self.function_name) )
	}
}
