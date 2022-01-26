// required for successful compilation ...
#[cfg(all(not(test), feature = "native_rust_rt"))]
pub mod rust_rt;
/// Services. Also visible to roottask, because some type definitions are shared.
pub mod services;
// Q&D: allow this always; currently roottask doesnt build otherwise, because stuff in
// services::* references it
// #[cfg(any(feature = "native_rust_rt", feature = "foreign_rust_rt"))]
#[cfg(feature = "foreign_rust_rt")]
pub mod hybrid_rt;
pub mod user_load_utcb;
#[cfg(feature = "native_rust_rt")]
pub mod user_logger;
