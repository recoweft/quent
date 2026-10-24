//! Owned, DB-free planning. A plan is never an execution permission.
//! Detailed material is available only through [`Plan::sensitive_source`].
mod plan;
pub use plan::*;
