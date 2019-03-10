//! Provides a few convenience extras for logging.
//! - FnGuard allow tracing entering and leaving functions
//! - ThreadLocalDrain adds a thread id to the structured logging
//!
mod fn_guard           ;
mod thread_local_drain ;

pub use           fn_guard :: FnGuard          ;
pub use thread_local_drain :: ThreadLocalDrain ;
