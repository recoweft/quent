//! Quent facade bootstrap. No DB adapter or execution API is implemented yet.
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
