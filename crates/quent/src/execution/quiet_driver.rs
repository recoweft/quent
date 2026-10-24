//! Scope the driver diagnostics to each poll AND cancellation/drop. A guard must
//! never live across Pending and affect unrelated tasks on the runtime thread.
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tracing::{Dispatch, dispatcher};
struct Quiet<F> {
    future: Option<Pin<Box<F>>>,
    dispatch: Dispatch,
}
pub(crate) fn quiet<F: Future>(future: F) -> impl Future<Output = F::Output> {
    Quiet {
        future: Some(Box::pin(future)),
        dispatch: Dispatch::new(tracing::subscriber::NoSubscriber::default()),
    }
}
impl<F: Future> Future for Quiet<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        dispatcher::with_default(&this.dispatch, || {
            this.future
                .as_mut()
                .expect("QUENT_DRIVER_FUTURE")
                .as_mut()
                .poll(cx)
        })
    }
}
impl<F> Drop for Quiet<F> {
    fn drop(&mut self) {
        dispatcher::with_default(&self.dispatch, || drop(self.future.take()));
    }
}

#[cfg(all(test, feature = "output-tracing"))]
use crate::output_capture;

#[cfg(all(test, feature = "output-tracing"))]
mod tests {
    use super::*;
    struct Hostile;
    impl Future for Hostile {
        type Output = ();
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
            let span = tracing::info_span!("CANARY_SPAN", source = "CANARY_SOURCE");
            let _entered = span.enter();
            tracing::info!(target:"CANARY_TARGET", source="CANARY_SQL_VALUE", "CANARY_MESSAGE");
            Poll::Pending
        }
    }
    impl Drop for Hostile {
        fn drop(&mut self) {
            tracing::info!("CANARY_DROP");
        }
    }
    #[test]
    fn driver_poll_and_drop_are_quiet_without_hiding_unrelated_events() {
        let capture = output_capture::Capture::default();
        tracing::subscriber::with_default(capture.clone(), || {
            let mut future = Box::pin(quiet(Hostile));
            let mut cx = Context::from_waker(std::task::Waker::noop());
            assert!(future.as_mut().poll(&mut cx).is_pending());
            let t = quent_core::Table::new(vec![quent_core::Ident::new("canary").unwrap()], None)
                .unwrap();
            let p = quent_core::Draft::delete(t)
                .freeze()
                .finalize(quent_core::Dialect::Postgres)
                .unwrap();
            crate::output_tracing::emit(&p.public_view()); // scope restored after Pending
            drop(future); // inner Drop is also scoped
        });
        capture.assert_safe(1);
        // Positive recorder control: hostile emissions really are observable.
        let control = output_capture::Capture::default();
        tracing::subscriber::with_default(control.clone(), || {
            let mut f = Box::pin(Hostile);
            let mut cx = Context::from_waker(std::task::Waker::noop());
            assert!(f.as_mut().poll(&mut cx).is_pending());
            drop(f);
        });
        assert!(
            control
                .0
                .lock()
                .unwrap()
                .iter()
                .any(|s| s.contains("CANARY_SOURCE"))
        );
        assert!(
            control
                .0
                .lock()
                .unwrap()
                .iter()
                .any(|s| s.contains("CANARY_DROP"))
        );
    }
}
