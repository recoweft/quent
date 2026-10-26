//! Application-only explicit authorization. Unbound acquisition stays non-executable;
//! runtime-bound candidates enter the one common dispatcher through linear permits.
//! The application authenticates subjects and supplies trusted adapters. Identity
//! labels and constructors do not authenticate servers, callers or providers.
use quent_core::*;
use std::{
    fmt,
    future::Future,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};

/// Closed failures with no provider text, source chain or operation data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationError {
    MissingProvider,
    MissingSubject,
    InvalidRegistration,
    InvalidTtl,
    Policy(PolicyError),
    Unsupported,
    Mismatch,
    Expired,
    Denied,
    ProviderError,
    Cancelled,
    Timeout,
}
impl fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_AUTHORIZATION_{self:?}")
    }
}
impl std::error::Error for AuthorizationError {}
type Result<T> = std::result::Result<T, AuthorizationError>;
macro_rules! redacted { ($($t:ty),+ $(,)?) => {$ (impl fmt::Debug for $t {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(concat!(stringify!($t), "([REDACTED])")) }
})+}; }

/// Finite supported request scopes, not arbitrary claims or PII reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationScope {
    MultiRowUpdate,
    MultiRowDelete,
}
/// Only a request can turn this finite response into a bound decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterResponse {
    AllowCandidate,
    Deny,
    Error,
    Cancelled,
    Timeout,
}

/// Trusted application callback. Receives sensitive complete preparation and
/// registration only through explicit getters; never an executor or transaction.
/// Implementations must not log sensitive inputs or panic with sensitive text.
/// No automatic retries, timeout task or external cancellation protocol is supplied.
pub trait TrustedAuthorizationAdapter: Send + Sync {
    fn authorize<'a>(
        &'a self,
        request: AuthorizationRequest<'a>,
    ) -> Pin<Box<dyn Future<Output = AuthorizationDecision> + Send + 'a>>;
}
struct Registration {
    identity: String,
    generation: u64,
    configuration: Vec<u8>,
    adapter: Box<dyn TrustedAuthorizationAdapter>,
}
/// Immutable per-database registration/snapshot. Clone shares only the exact
/// immutable registration instance, deep-copying the Core snapshot. Adapter
/// implementation interior state is the application's trusted responsibility.
#[derive(Clone)]
pub struct AuthorizationContext {
    database: DatabaseContext,
    registration: Option<Arc<Registration>>,
}
impl AuthorizationContext {
    pub fn without_provider(database: DatabaseContext) -> Self {
        Self {
            database,
            registration: None,
        }
    }
    /// Full bounded configuration bytes are retained, not a hash. They can be
    /// sensitive. No authentication or remote configuration verification occurs.
    pub fn register(
        database: DatabaseContext,
        identity: String,
        generation: u64,
        configuration: Vec<u8>,
        adapter: impl TrustedAuthorizationAdapter + 'static,
    ) -> Result<Self> {
        if identity.is_empty()
            || identity.len() > 128
            || !identity
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-.:/".contains(&b))
            || generation == 0
            || configuration.len() > 4096
        {
            return Err(AuthorizationError::InvalidRegistration);
        }
        if database.sensitive_source().subject().is_none() {
            return Err(AuthorizationError::MissingSubject);
        }
        Ok(Self {
            database,
            registration: Some(Arc::new(Registration {
                identity,
                generation,
                configuration,
                adapter: Box::new(adapter),
            })),
        })
    }
    /// Explicit derivation inherits the same provider/configuration instance.
    /// Target/routing/catalog/dialect must stay exact. New subject/tenant/schema/
    /// settings are explicit Core snapshots, never silently taken from siblings.
    pub fn with_database(&self, database: DatabaseContext) -> Result<Self> {
        if self.database.sensitive_source().target() != database.sensitive_source().target() {
            return Err(AuthorizationError::Mismatch);
        }
        if database.sensitive_source().subject().is_none() {
            return Err(AuthorizationError::MissingSubject);
        }
        Ok(Self {
            database,
            registration: self.registration.clone(),
        })
    }
    pub fn sensitive_database(&self) -> &DatabaseContext {
        &self.database
    }
    pub(crate) fn matches(&self, other: &Self) -> bool {
        self.database.same_snapshot(&other.database)
            && match (&self.registration, &other.registration) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
    /// DB/provider-free preparation, retaining the sole finalized record.
    pub fn candidate(
        &self,
        operation: &Operation,
        scope: AuthorizationScope,
        ttl: Duration,
    ) -> Result<AuthorizationCandidate> {
        self.candidate_at(operation, scope, ttl, Instant::now())
    }
    fn candidate_at(
        &self,
        operation: &Operation,
        scope: AuthorizationScope,
        ttl: Duration,
        now: Instant,
    ) -> Result<AuthorizationCandidate> {
        if ttl.is_zero() || ttl > Duration::from_secs(300) {
            return Err(AuthorizationError::InvalidTtl);
        }
        let expires = now.checked_add(ttl).ok_or(AuthorizationError::InvalidTtl)?;
        if self.registration.is_none() {
            return Err(AuthorizationError::MissingProvider);
        }
        if self.database.sensitive_source().subject().is_none() {
            return Err(AuthorizationError::MissingSubject);
        }
        let prepared = self
            .database
            .prepare()
            .prepare_operation(operation)
            .map_err(AuthorizationError::Policy)?;
        eligible(&prepared, scope)?;
        Ok(AuthorizationCandidate {
            prepared,
            context: self.clone(),
            scope,
            expires,
            runtime: None,
        })
    }
}
pub(crate) fn eligible(p: &PolicyPreparation, scope: AuthorizationScope) -> Result<()> {
    let i = p.inspect();
    let source = p.sensitive_source();
    let final_source = source.final_source();
    let metadata = final_source.metadata();
    // No INSERT, SELECT, Partial/Unknown, MissingWhere, RETURNING or version
    // expansion. Native encoding is the existing finite Core value domain.
    let family = match scope {
        AuthorizationScope::MultiRowUpdate => Family::Update,
        AuthorizationScope::MultiRowDelete => Family::Delete,
    };
    if metadata.family() != family
        || metadata.optimistic_version()
        || !matches!(metadata.projection(), FinalProjection::None)
        || i.completeness() != Completeness::Complete
        || !matches!(i.scope(), Scope::Broad | Scope::FiniteBound(_))
        || !matches!(i.risk(), Risk::High | Risk::Destructive)
    {
        return Err(AuthorizationError::Unsupported);
    }
    Ok(())
}
/// Linear, private exact preparation. Moving into acquisition consumes it even
/// when the returned future is never polled or is dropped. No execution method.
pub struct AuthorizationCandidate {
    prepared: PolicyPreparation,
    context: AuthorizationContext,
    scope: AuthorizationScope,
    expires: Instant,
    runtime: Option<RuntimeBinding>,
}
/// Private execution identity, installed before the provider sees the request.
#[cfg_attr(
    not(all(
        feature = "runtime-tokio",
        any(feature = "postgres", feature = "mysql")
    )),
    allow(dead_code)
)]
pub(crate) struct RuntimeBinding {
    pub(crate) session: Arc<()>,
    pub(crate) request: Arc<()>,
}
#[cfg(all(
    feature = "runtime-tokio",
    any(feature = "postgres", feature = "mysql")
))]
impl AuthorizationCandidate {
    pub(crate) fn check_runtime(&self, session: &Arc<()>, request: &Arc<()>) -> Result<()> {
        match &self.runtime {
            Some(b) if Arc::ptr_eq(&b.session, session) && Arc::ptr_eq(&b.request, request) => {
                Ok(())
            }
            _ => Err(AuthorizationError::Mismatch),
        }
    }
    pub(crate) fn bind_runtime(&mut self, session: Arc<()>, request: Arc<()>) {
        self.runtime = Some(RuntimeBinding { session, request });
    }
    pub(crate) fn into_execution(
        self,
    ) -> (
        PolicyPreparation,
        AuthorizationContext,
        AuthorizationScope,
        Instant,
        RuntimeBinding,
    ) {
        (
            self.prepared,
            self.context,
            self.scope,
            self.expires,
            self.runtime.expect("bound internal candidate"),
        )
    }
}
/// Borrowed sensitive review input. `respond` consumes this one request wrapper;
/// the private invocation identity cannot be copied or manufactured by adapters.
pub struct AuthorizationRequest<'a> {
    candidate: &'a AuthorizationCandidate,
    invocation: Arc<()>,
}
impl AuthorizationRequest<'_> {
    /// Confidential: SQL, ordered values, metadata, original operation, subject,
    /// tenant, routing and complete settings/policy/schema assertions.
    pub fn sensitive_preparation(&self) -> &PolicyPreparation {
        &self.candidate.prepared
    }
    /// Confidential full registration, not proof of external service identity.
    pub fn sensitive_registration(&self) -> (&str, u64, &[u8]) {
        let r = self.candidate.context.registration.as_ref().unwrap();
        (&r.identity, r.generation, &r.configuration)
    }
    /// Whether an execution request/session was retained before this callback.
    /// This is not authentication of an HTTP request or physical server.
    pub fn is_runtime_bound(&self) -> bool {
        self.candidate.runtime.is_some()
    }
    pub fn scope(&self) -> AuthorizationScope {
        self.candidate.scope
    }
    pub fn deadline(&self) -> Instant {
        self.candidate.expires
    }
    pub fn respond(self, response: AdapterResponse) -> AuthorizationDecision {
        AuthorizationDecision {
            invocation: self.invocation,
            response,
        }
    }
}
/// Provider response privately bound to exactly one invocation. Not a permit.
pub struct AuthorizationDecision {
    invocation: Arc<()>,
    response: AdapterResponse,
}
/// Consumed successful acquisition record. PR1 deliberately has no dispatch,
/// permit conversion, getter, serialization or public restoration interface.
pub struct AuthorizationAcquisition {
    _candidate: AuthorizationCandidate,
}
redacted!(
    AuthorizationContext,
    AuthorizationCandidate,
    AuthorizationRequest<'_>,
    AuthorizationDecision,
    AuthorizationAcquisition
);

fn check_bindings(
    candidate: &AuthorizationCandidate,
    current: &AuthorizationContext,
) -> Result<()> {
    if !candidate.context.matches(current) {
        return Err(AuthorizationError::Mismatch);
    }
    eligible(&candidate.prepared, candidate.scope)
}
fn check_expiry(candidate: &AuthorizationCandidate, now: Instant) -> Result<()> {
    if now >= candidate.expires {
        return Err(AuthorizationError::Expired);
    }
    Ok(())
}
/// Compatibility non-executing acquisition API. Both snapshots are immutable for this borrow;
/// independently changed application routing/session state is not observable here.
/// This unbound receipt has no executable conversion. An adapter that never
/// completes is not automatically timed out; dropping cancels local polling only.
pub async fn acquire_authorization(
    candidate: AuthorizationCandidate,
    current: &AuthorizationContext,
) -> Result<AuthorizationAcquisition> {
    let candidate = acquire_candidate(candidate, current).await?;
    Ok(AuthorizationAcquisition {
        _candidate: candidate,
    })
}
pub(crate) async fn acquire_candidate(
    candidate: AuthorizationCandidate,
    current: &AuthorizationContext,
) -> Result<AuthorizationCandidate> {
    check_bindings(&candidate, current)?;
    check_expiry(&candidate, Instant::now())?;
    let r = candidate
        .context
        .registration
        .as_ref()
        .ok_or(AuthorizationError::MissingProvider)?;
    let invocation = Arc::new(());
    let decision = r
        .adapter
        .authorize(AuthorizationRequest {
            candidate: &candidate,
            invocation: invocation.clone(),
        })
        .await;
    check_bindings(&candidate, current)?;
    if !Arc::ptr_eq(&invocation, &decision.invocation) {
        return Err(AuthorizationError::Mismatch);
    }
    // Sample after *all* bounded comparisons, never before their work.
    check_expiry(&candidate, Instant::now())?;
    match decision.response {
        AdapterResponse::AllowCandidate => Ok(candidate),
        AdapterResponse::Deny => Err(AuthorizationError::Denied),
        AdapterResponse::Error => Err(AuthorizationError::ProviderError),
        AdapterResponse::Cancelled => Err(AuthorizationError::Cancelled),
        AdapterResponse::Timeout => Err(AuthorizationError::Timeout),
    }
}

#[cfg(test)]
mod tests;
