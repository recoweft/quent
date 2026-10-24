//! Only sealed public summaries can enter this emission. No dynamic event text,
//! Display/Error callback, source object, span field or Debug record is accepted.
use quent_core::{DiagnosticView, PlanView};
mod sealed {
    pub trait Sealed {}
}
pub trait PublicSummary: sealed::Sealed {
    #[doc(hidden)]
    fn public_json(&self) -> String;
}
macro_rules! summary {
    ($t:ty) => {
        impl sealed::Sealed for $t {}
        impl PublicSummary for $t {
            fn public_json(&self) -> String {
                self.to_json()
            }
        }
    };
}
summary!(PlanView);
summary!(DiagnosticView);
#[cfg(all(
    feature = "runtime-tokio",
    any(feature = "postgres", feature = "mysql")
))]
summary!(crate::ResultView);
/// Emits exactly the documentary public-view JSON schema. A subscriber supplied
/// by the application can add its own metadata; Quent cannot sanitize that data.
pub fn emit(summary: &impl PublicSummary) {
    tracing::event!(name: "quent_public_view", target: "quent::public", tracing::Level::INFO,
        public_view = %summary.public_json());
}
