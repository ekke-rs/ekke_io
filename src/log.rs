//! Provides a few convenience extras for logging.
//! - FnGuard allow tracing entering and leaving functions
//! - ThreadLocalDrain adds a thread id to the structured logging
//! - ResultExtSlog allows to call unwraps( log: slog::Logger ) which will log the error before unwrapping.
//!
mod fn_guard           ;
mod thread_local_drain ;
mod result_ext_slog    ;

pub use           fn_guard :: FnGuard          ;
pub use    result_ext_slog :: ResultExtSlog    ;
pub use thread_local_drain :: ThreadLocalDrain ;
