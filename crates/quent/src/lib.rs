//! Immediate application-only execution of policy-prepared operations.
//!
//! Application snapshots have no public JSON restoration or export boundary.
//! These examples use the test-only serde dependency to check the actual traits.
//! ```compile_fail
//! let context: quent_core::ApplicationContext =
//!     serde_json::from_str(r#"{"tenant":1,"fresh":"live","allow":true}"#).unwrap();
//! ```
//! ```compile_fail
//! fn public_json(context: &quent_core::DatabaseContext) {
//!     serde_json::to_string(context).unwrap();
//! }
//! ```
//!
//! Policy preparation is neither serializable authority nor an executable handle.
//! ```compile_fail
//! fn restore() {
//!     let _: quent_core::PolicyPreparation = serde_json::from_str("{}").unwrap();
//! }
//! ```
//! ```compile_fail
//! fn export(p: &quent_core::PolicyPreparation) { serde_json::to_string(p).unwrap(); }
//! ```

#[cfg(all(
    feature = "runtime-tokio",
    any(feature = "postgres", feature = "mysql")
))]
mod execution;
#[cfg(all(
    feature = "runtime-tokio",
    any(feature = "postgres", feature = "mysql")
))]
pub use execution::*;

/// Explicit local-only output. Never an input/execution interface.
#[cfg(feature = "local-details")]
pub mod local_details;
/// Closed public tracing emissions; installs no subscriber or SQL logger.
#[cfg(feature = "output-tracing")]
pub mod output_tracing;

#[cfg(all(test, feature = "output-tracing"))]
extern crate self as quent;

#[cfg(all(test, feature = "output-tracing"))]
#[path = "../tests/output_capture/mod.rs"]
mod output_capture;
