//! Explicit human-controlled, application-attested finite migration application.
//! No public wire, CRUD permit, reader evidence or arbitrary SQL enters this gate.
use super::preflight::{PreflightObservation, PreflightRequest};
use super::*;
use std::{future::Future, pin::Pin};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyError {
    Invalid,
    MissingAdapter,
    MissingSubject,
    Mismatch,
    Expired,
    Refused,
    Attestation,
    HumanDenied,
    Observation,
    Unavailable,
}
impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_MIGRATION_{self:?}")
    }
}
impl std::error::Error for ApplyError {}
pub(super) type ApplyResult<T> = std::result::Result<T, ApplyError>;

/// Five independent residual requirements. None is a live observation by Quent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    AuthenticatedRouting,
    CompleteEffectsAndIncomingReferences,
    DeclarationProvenance,
    DdlPrivilegesAndVisibility,
    ContinuousDdlExclusion,
}
const REQUIREMENTS: [Requirement; 5] = [
    Requirement::AuthenticatedRouting,
    Requirement::CompleteEffectsAndIncomingReferences,
    Requirement::DeclarationProvenance,
    Requirement::DdlPrivilegesAndVisibility,
    Requirement::ContinuousDdlExclusion,
];
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttestedState {
    ApplicationAttested,
    Unknown,
    Denied,
}
/// Bounded validity, exact generation and finite state of one application assertion.
/// Constructors do not authenticate facts or people. The trusted adapter must
/// establish the assertion's basis for the complete sensitive request.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Assertion {
    pub(super) state: AttestedState,
    pub(super) generation: std::num::NonZeroU64,
    pub(super) expires: Instant,
}
impl Assertion {
    pub fn sensitive_parts(&self) -> (AttestedState, std::num::NonZeroU64, Instant) {
        (self.state, self.generation, self.expires)
    }
    pub fn new(state: AttestedState, generation: std::num::NonZeroU64, expires: Instant) -> Self {
        Self {
            state,
            generation,
            expires,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanState {
    ExplicitlyAuthorized,
    Unknown,
    Denied,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct HumanInstruction {
    pub(super) state: HumanState,
    pub(super) generation: std::num::NonZeroU64,
    pub(super) expires: Instant,
}
impl HumanInstruction {
    pub fn sensitive_parts(&self) -> (HumanState, std::num::NonZeroU64, Instant) {
        (self.state, self.generation, self.expires)
    }
    /// The application authenticates a person's explicit operation and intent.
    /// This value alone is not an approval; a request-bound adapter response is required.
    pub fn new(state: HumanState, generation: std::num::NonZeroU64, expires: Instant) -> Self {
        Self {
            state,
            generation,
            expires,
        }
    }
}
/// Non-authoritative allocation identity for independent current snapshots.
#[derive(Clone)]
pub struct SessionIdentity(pub(super) Arc<()>);
pub(super) struct Registration {
    pub(super) identity: String,
    pub(super) generation: std::num::NonZeroU64,
    pub(super) configuration: Vec<u8>,
    pub(super) adapter: Box<dyn TrustedMigrationAdapter>,
}
#[derive(Clone)]
pub struct ApplyContext {
    pub(super) database: DatabaseContext,
    pub(super) registration: Option<Arc<Registration>>,
}
impl ApplyContext {
    pub fn without_adapter(database: DatabaseContext) -> Self {
        Self {
            database,
            registration: None,
        }
    }
    pub fn register(
        database: DatabaseContext,
        identity: String,
        generation: std::num::NonZeroU64,
        configuration: Vec<u8>,
        adapter: impl TrustedMigrationAdapter + 'static,
    ) -> ApplyResult<Self> {
        if identity.is_empty()
            || identity.len() > 128
            || configuration.len() > 4096
            || !identity
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-.:/".contains(&b))
        {
            return Err(ApplyError::Invalid);
        }
        if database.sensitive_source().subject().is_none() {
            return Err(ApplyError::MissingSubject);
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
    pub fn sensitive_database(&self) -> &DatabaseContext {
        &self.database
    }
    /// Preserve the immutable registration, explicitly replace application context.
    pub fn with_database(&self, database: DatabaseContext) -> ApplyResult<Self> {
        if self.database.sensitive_source().target() != database.sensitive_source().target() {
            return Err(ApplyError::Mismatch);
        }
        Ok(Self {
            database,
            registration: self.registration.clone(),
        })
    }
    fn same(&self, other: &Self) -> bool {
        self.database.same_snapshot(&other.database)
            && match (&self.registration, &other.registration) {
                (Some(a), Some(b)) => {
                    Arc::ptr_eq(a, b)
                        && a.identity == b.identity
                        && a.generation == b.generation
                        && a.configuration == b.configuration
                }
                (None, None) => true,
                _ => false,
            }
    }
}
/// One application-created human operation identity, not authentication or a
/// permission. Reconstructing the same textual ID creates a different operation.
/// Keep this object (or an immutable Arc to it) for independent current supply.
pub struct ApplyRequest {
    identity: Arc<()>,
    id: String,
}
impl ApplyRequest {
    pub fn new(id: String) -> ApplyResult<Self> {
        if id.is_empty() || id.len() > 128 || id.chars().any(char::is_control) {
            return Err(ApplyError::Invalid);
        }
        Ok(Self {
            identity: Arc::new(()),
            id,
        })
    }
}
/// Independent current application facts. Exact plan and request are retained;
/// matching labels or epochs alone never decide correspondence.
pub struct Current<'p> {
    pub(super) context: ApplyContext,
    pub(super) session: SessionIdentity,
    pub(super) request_id: String,
    pub(super) request_identity: Arc<()>,
    pub(super) request: PreflightRequest<'p>,
    pub(super) assertions: [Assertion; 5],
    pub(super) human: HumanInstruction,
}
impl<'p> Current<'p> {
    pub fn new(
        context: ApplyContext,
        session: SessionIdentity,
        operation: &ApplyRequest,
        request: PreflightRequest<'p>,
        assertions: [Assertion; 5],
        human: HumanInstruction,
    ) -> ApplyResult<Self> {
        Ok(Self {
            context,
            session,
            request_id: operation.id.clone(),
            request_identity: operation.identity.clone(),
            request,
            assertions,
            human,
        })
    }
    #[cfg(test)]
    fn retained(&self) -> Self {
        Self {
            context: self.context.clone(),
            session: self.session.clone(),
            request_id: self.request_id.clone(),
            request_identity: self.request_identity.clone(),
            request: self.request.retained(),
            assertions: self.assertions,
            human: self.human,
        }
    }
    fn same(&self, other: &Self) -> bool {
        self.context.same(&other.context)
            && Arc::ptr_eq(&self.session.0, &other.session.0)
            && Arc::ptr_eq(&self.request_identity, &other.request_identity)
            && self.request_id == other.request_id
            && self.request.same(&other.request)
            && self.assertions == other.assertions
            && self.human == other.human
    }
    pub(super) fn valid(&self, deadline: Instant) -> ApplyResult<()> {
        if self.context.registration.is_none() {
            return Err(ApplyError::MissingAdapter);
        }
        if self.context.database.sensitive_source().subject().is_none() {
            return Err(ApplyError::MissingSubject);
        }
        if self
            .assertions
            .iter()
            .any(|a| a.state != AttestedState::ApplicationAttested)
        {
            return Err(ApplyError::Attestation);
        }
        if self.human.state != HumanState::ExplicitlyAuthorized {
            return Err(ApplyError::HumanDenied);
        }
        // Clock sampled after full comparisons and finite validity checks.
        let now = Instant::now();
        if now >= deadline
            || now >= self.human.expires
            || self.assertions.iter().any(|a| now >= a.expires)
        {
            return Err(ApplyError::Expired);
        }
        Ok(())
    }
}
/// Synchronous independent current-state source. Implementations must return
/// genuinely current bounded snapshots, maintain exclusion until actual server
/// completion, and never replay retained permits as current. No executor is supplied.
pub trait CurrentSource<'p> {
    fn current(&self) -> ApplyResult<Current<'p>>;
}
/// Trusted in-process boundary, separate from CRUD authorization. Implementations
/// authenticate human intent and the five individual facts for this exact request.
/// External authentication, transport, logging and panic hygiene are application responsibilities.
pub trait TrustedMigrationAdapter: Send + Sync {
    fn authorize<'a>(
        &'a self,
        request: ApprovalRequest<'a>,
    ) -> Pin<Box<dyn Future<Output = ApprovalDecision> + Send + 'a>>;
}
pub struct ApprovalRequest<'a> {
    pub(super) current: &'a Current<'a>,
    pub(super) deadline: Instant,
    pub(super) invocation: Arc<()>,
}
impl ApprovalRequest<'_> {
    pub fn sensitive_plan(&self) -> &quent_core::migration::MigrationPlan {
        self.current.request.plan
    }
    pub fn sensitive_context(&self) -> &DatabaseContext {
        &self.current.context.database
    }
    /// Match the retained operation allocation as well as its full textual ID.
    /// This does not authenticate an external request or person.
    pub fn matches_operation(&self, operation: &ApplyRequest) -> bool {
        Arc::ptr_eq(&self.current.request_identity, &operation.identity)
            && self.current.request_id == operation.id
    }
    pub fn sensitive_request_id(&self) -> &str {
        &self.current.request_id
    }
    pub fn sensitive_registration(&self) -> (&str, std::num::NonZeroU64, &[u8]) {
        let r = self
            .current
            .context
            .registration
            .as_ref()
            .expect("registered");
        (&r.identity, r.generation, &r.configuration)
    }
    pub fn requirements(&self) -> &[Requirement; 5] {
        &REQUIREMENTS
    }
    pub fn sensitive_assertions(&self) -> &[Assertion; 5] {
        &self.current.assertions
    }
    pub fn sensitive_human_instruction(&self) -> HumanInstruction {
        self.current.human
    }
    pub fn sensitive_scope(
        &self,
    ) -> (
        &str,
        &[String],
        &str,
        std::num::NonZeroU64,
        super::preflight::DdlExclusion,
    ) {
        let r = &self.current.request;
        (
            &r.scope.namespace,
            &r.scope.tables,
            &r.visibility.role,
            r.visibility.generation,
            r.exclusion,
        )
    }
    pub fn phase(&self) -> quent_core::migration::MigrationPhase {
        self.current.request.phase
    }
    pub fn session_identity(&self) -> SessionIdentity {
        self.current.session.clone()
    }
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    /// Each supplied item is checked independently against current facts. The
    /// private invocation binds this decision; neither a boolean nor saved JSON works.
    pub fn respond(self, assertions: [Assertion; 5], human: HumanInstruction) -> ApprovalDecision {
        ApprovalDecision {
            invocation: self.invocation,
            assertions,
            human,
        }
    }
}
pub struct ApprovalDecision {
    pub(super) invocation: Arc<()>,
    pub(super) assertions: [Assertion; 5],
    pub(super) human: HumanInstruction,
}
/// Linear request, bound before any observation or adapter invocation.
pub struct ApplyCandidate<'p> {
    pub(super) current: Current<'p>,
    pub(super) deadline: Instant,
}
/// Linear private evidence + separate application and human decision. No restoration.
pub struct ApplyHandle<'p> {
    pub(super) candidate: ApplyCandidate<'p>,
    pub(super) observation: PreflightObservation<'p>,
}
impl ApplyCandidate<'_> {
    pub(super) fn check<'p>(&self, source: &impl CurrentSource<'p>) -> ApplyResult<()> {
        let current = source.current()?;
        if !self.current.same(&current) {
            return Err(ApplyError::Mismatch);
        }
        current.valid(self.deadline)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    NotSent,
    BeginUnknown,
    StepUnknown,
    StepRejected,
    CommitUnknown,
    RollbackUnknown,
    RolledBack,
    Committed,
    CommitAcknowledgedCurrentInvalid,
    ImplicitCommitPrefix,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceRecord {
    NotAcquired,
    FiniteObservationAndApplicationDecision,
}
/// Attempt is a driver call boundary, not proof bytes reached the server. ACK
/// prefix is not durable success for PG before COMMIT. Unknown is never retried.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplyOutcome {
    pub evidence_record: EvidenceRecord,
    pub completion: Completion,
    pub steps: usize,
    pub attempted: usize,
    pub acknowledged: usize,
    pub unacknowledged_step: Option<usize>,
    pub rejected_step: Option<usize>,
    pub unsent: usize,
    pub refusal: Option<ApplyError>,
}
impl ApplyOutcome {
    /// Output only; partial live coverage and application trust are distinct.
    pub fn to_json(self) -> String {
        let index = |v: Option<usize>| v.map_or_else(|| "null".to_owned(), |i| i.to_string());
        let refusal = self
            .refusal
            .map_or_else(|| "null".to_owned(), |e| format!("\"{e:?}\""));
        format!(
            "{{\"kind\":\"quent_migration_apply_view\",\"version\":1,\"live_coverage\":\"Partial\",\"residual_trust_source\":\"RegisteredApplicationAdapter\",\"human_trust_source\":\"ApplicationAdapter\",\"evidence_record\":\"{:?}\",\"full_effects\":\"UnverifiedByQuent\",\"completion\":\"{:?}\",\"steps\":{},\"attempted\":{},\"acknowledged\":{},\"unacknowledged_step\":{},\"rejected_step\":{},\"unsent\":{},\"refusal\":{}}}",
            self.evidence_record,
            self.completion,
            self.steps,
            self.attempted,
            self.acknowledged,
            index(self.unacknowledged_step),
            index(self.rejected_step),
            self.unsent,
            refusal
        )
    }
    pub(super) fn new(steps: usize) -> Self {
        Self {
            evidence_record: EvidenceRecord::NotAcquired,
            completion: Completion::NotSent,
            steps,
            attempted: 0,
            acknowledged: 0,
            unacknowledged_step: None,
            rejected_step: None,
            unsent: steps,
            refusal: None,
        }
    }
}
impl fmt::Display for ApplyOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
redacted!(
    Assertion,
    HumanInstruction,
    SessionIdentity,
    ApplyContext,
    ApplyRequest,
    Current<'_>,
    ApprovalRequest<'_>,
    ApprovalDecision,
    ApplyCandidate<'_>,
    ApplyHandle<'_>
);

// Internal polling guard: SQLx can suspend for preparation/protocol I/O before
// sending Execute. Recheck independent current state on every resume as well as
// the explicit statement boundaries. No driver/future is exposed to applications.
pub(super) async fn guarded<F: Future>(
    future: F,
    check: impl Fn() -> ApplyResult<()>,
) -> ApplyResult<F::Output> {
    let mut future = Box::pin(future);
    std::future::poll_fn(|cx| {
        if let Err(e) = check() {
            return std::task::Poll::Ready(Err(e));
        }
        match future.as_mut().poll(cx) {
            std::task::Poll::Ready(value) => std::task::Poll::Ready(Ok(value)),
            std::task::Poll::Pending => match check() {
                Ok(()) => std::task::Poll::Pending,
                Err(e) => std::task::Poll::Ready(Err(e)),
            },
        }
    })
    .await
}
// Expanded only in closed SQLx adapters. The wrapper privately owns a freshly
// connected reader; no existing reader/evidence can be passed to this API.
macro_rules! apply_session {
    ($session:ident, $reader:ident, $options:ty, $pg:expr) => {
        pub struct $session {
            pub(super) reader: $reader,
            progress: super::apply::ApplyOutcome,
        }
        redacted!($session);
        impl $session {
            pub async fn connect(
                options: $options,
                context: DatabaseContext,
                timeout: Duration,
            ) -> super::apply::ApplyResult<Self> {
                Ok(Self {
                    reader: $reader::connect(options, context, timeout)
                        .await
                        .map_err(|_| super::apply::ApplyError::Unavailable)?,
                    progress: super::apply::ApplyOutcome::new(0),
                })
            }
            /// Last finite progress, retained even when a polled future is dropped.
            /// Unknown remains unknown until independent human reconciliation.
            pub fn progress(&self) -> super::apply::ApplyOutcome {
                self.progress
            }
            pub fn identity(&self) -> super::apply::SessionIdentity {
                super::apply::SessionIdentity(self.reader.identity.clone())
            }
            pub fn candidate<'p>(
                &self,
                current: super::apply::Current<'p>,
                ttl: Duration,
            ) -> super::apply::ApplyResult<super::apply::ApplyCandidate<'p>> {
                use super::apply::*;
                let mut deadline = expiry(Instant::now(), ttl).map_err(|_| ApplyError::Expired)?;
                deadline = deadline.min(current.human.expires);
                for a in current.assertions {
                    deadline = deadline.min(a.expires);
                }
                if !self.reader.available
                    || !Arc::ptr_eq(&current.session.0, &self.reader.identity)
                    || !current.context.database.same_snapshot(&self.reader.context)
                    || current
                        .request
                        .expected()
                        .map_err(|_| ApplyError::Refused)?
                        .target
                        != self.reader.target
                {
                    return Err(ApplyError::Mismatch);
                }
                current.valid(deadline)?;
                Ok(ApplyCandidate { current, deadline })
            }
            pub async fn prepare<'p>(
                &mut self,
                candidate: super::apply::ApplyCandidate<'p>,
                source: &impl super::apply::CurrentSource<'p>,
                budget: Budget,
            ) -> super::apply::ApplyResult<super::apply::ApplyHandle<'p>> {
                use super::apply::*;
                candidate.check(source)?;
                if !self.reader.available
                    || !Arc::ptr_eq(&candidate.current.session.0, &self.reader.identity)
                    || !candidate
                        .current
                        .context
                        .database
                        .same_snapshot(&self.reader.context)
                {
                    return Err(ApplyError::Mismatch);
                }
                // No public reader evidence enters. This new observation has this
                // exact connection identity, with ACK read-only rollback first.
                let observation = self
                    .reader
                    .observe_migration(
                        &candidate.current.context.database,
                        &candidate.current.request,
                        budget,
                        candidate.deadline.saturating_duration_since(Instant::now()),
                    )
                    .await
                    .map_err(|_| ApplyError::Observation)?;
                self.reader.available = false;
                candidate.check(source)?;
                let r = candidate
                    .current
                    .context
                    .registration
                    .as_ref()
                    .ok_or(ApplyError::MissingAdapter)?;
                let invocation = Arc::new(());
                #[cfg(test)]
                self.reader
                    .trace
                    .lock()
                    .unwrap()
                    .push("migration_provider_attempt");
                let decision = quiet(tokio::time::timeout_at(
                    tokio::time::Instant::from_std(candidate.deadline),
                    r.adapter.authorize(ApprovalRequest {
                        current: &candidate.current,
                        deadline: candidate.deadline,
                        invocation: invocation.clone(),
                    }),
                ))
                .await
                .map_err(|_| ApplyError::Expired)?;
                #[cfg(test)]
                self.reader
                    .trace
                    .lock()
                    .unwrap()
                    .push("migration_provider_ack");
                candidate.check(source)?;
                if !Arc::ptr_eq(&invocation, &decision.invocation)
                    || decision.assertions != candidate.current.assertions
                    || decision.human != candidate.current.human
                {
                    return Err(ApplyError::Mismatch);
                }
                // Lifetime does not exceed any individual assertion/approval/observation.
                let mut candidate = candidate;
                candidate.deadline = candidate
                    .deadline
                    .min(observation.deadline)
                    .min(decision.human.expires);
                for a in decision.assertions {
                    candidate.deadline = candidate.deadline.min(a.expires);
                }
                candidate.check(source)?;
                self.reader.available = true;
                Ok(ApplyHandle {
                    candidate,
                    observation,
                })
            }
            /// Ownership is consumed at future construction, even if never polled.
            /// Any polled application quarantines this session permanently.
            pub async fn apply<'p>(
                &mut self,
                handle: super::apply::ApplyHandle<'p>,
                source: &impl super::apply::CurrentSource<'p>,
            ) -> super::apply::ApplyOutcome {
                use super::apply::*;
                let sql = handle.candidate.current.request.plan.sensitive_source();
                let mut out = ApplyOutcome::new(sql.sql().len());
                out.evidence_record = EvidenceRecord::FiniteObservationAndApplicationDecision;
                if !self.reader.available
                    || !Arc::ptr_eq(&handle.candidate.current.session.0, &self.reader.identity)
                    || !handle
                        .candidate
                        .current
                        .context
                        .database
                        .same_snapshot(&self.reader.context)
                {
                    out.refusal = Some(ApplyError::Mismatch);
                    return out;
                }
                let check = || -> ApplyResult<()> {
                    handle.candidate.check(source)?;
                    if handle
                        .observation
                        .compare(
                            true,
                            &self.reader.identity,
                            &handle.candidate.current.context.database,
                            &handle.candidate.current.request,
                        )
                        .state()
                        != super::preflight::PreflightState::ConditionsObserved
                    {
                        return Err(ApplyError::Observation);
                    }
                    handle.candidate.current.valid(handle.candidate.deadline)
                };
                if let Err(e) = check() {
                    out.refusal = Some(e);
                    return out;
                }
                self.reader.available = false;
                self.progress = out;
                let Some(connection) = self.reader.connection.as_mut() else {
                    out.refusal = Some(ApplyError::Unavailable);
                    return out;
                };
                let deadline = tokio::time::Instant::from_std(handle.candidate.deadline);
                if $pg {
                    if let Err(e) = check() {
                        out.refusal = Some(e);
                        return out;
                    }
                    #[cfg(test)]
                    self.reader
                        .trace
                        .lock()
                        .unwrap()
                        .push("apply_begin_attempt");
                    self.progress = out;
                    self.progress.completion = Completion::BeginUnknown;
                    let result = quiet(tokio::time::timeout_at(
                        deadline,
                        guarded(sqlx::query("BEGIN").execute(&mut *connection), &check),
                    ))
                    .await
                    .map_err(|_| ApplyError::Expired)
                    .and_then(|r| r);
                    if let Err(e) = &result {
                        out.refusal = Some(*e);
                    }
                    let after_begin = check();
                    if !matches!(result, Ok(Ok(_))) {
                        out.completion = Completion::BeginUnknown;
                        if let Err(e) = after_begin {
                            out.refusal = Some(e);
                        }
                        self.progress = out;
                        return out;
                    }
                    #[cfg(test)]
                    self.reader.trace.lock().unwrap().push("apply_begin_ack");
                    out.completion = Completion::RollbackUnknown;
                    if let Err(e) = after_begin {
                        out.refusal = Some(e);
                    }
                }
                for (index, statement) in sql.sql().iter().enumerate() {
                    if out.refusal.is_some() {
                        break;
                    }
                    if let Err(e) = check() {
                        out.refusal = Some(e);
                        break;
                    }
                    out.attempted += 1;
                    out.unsent -= 1;
                    out.unacknowledged_step = Some(index);
                    #[cfg(test)]
                    self.reader.trace.lock().unwrap().push("ddl_attempt");
                    self.progress = out;
                    self.progress.completion = Completion::StepUnknown;
                    let result = quiet(tokio::time::timeout_at(
                        deadline,
                        guarded(
                            sqlx::query(sqlx::AssertSqlSafe(statement.clone()))
                                .execute(&mut *connection),
                            &check,
                        ),
                    ))
                    .await
                    .map_err(|_| ApplyError::Expired)
                    .and_then(|r| r);
                    if let Err(e) = &result {
                        out.refusal = Some(*e);
                    }
                    // Each await is followed by independent current/full comparisons,
                    // even when driver result is error/unknown.
                    let current_result = check();
                    match result {
                        Ok(Ok(result)) if result.rows_affected() == 0 => {
                            out.acknowledged += 1;
                            out.unacknowledged_step = None;
                            #[cfg(test)]
                            self.reader.trace.lock().unwrap().push("ddl_ack");
                        }
                        Ok(Err(error)) if error.as_database_error().is_some() => {
                            out.completion = Completion::StepRejected;
                            out.rejected_step = Some(index);
                            out.unacknowledged_step = None;
                            #[cfg(test)]
                            self.reader.trace.lock().unwrap().push("ddl_rejected");
                            if let Err(e) = current_result {
                                out.refusal = Some(e);
                            }
                            break;
                        }
                        _ => {
                            out.completion = Completion::StepUnknown;
                            if let Err(e) = current_result {
                                out.refusal = Some(e);
                            }
                            break;
                        }
                    }
                    if let Err(e) = current_result {
                        out.refusal = Some(e);
                        break;
                    }
                }
                if $pg {
                    let ready = out.acknowledged == out.steps
                        && out.refusal.is_none()
                        && out.unacknowledged_step.is_none();
                    let current = check();
                    if let Err(e) = current {
                        out.refusal = Some(e);
                    }
                    let commit = ready && out.refusal.is_none();
                    // Loss of current authorization never authorizes more DDL or
                    // COMMIT. Explicit rollback cleanup may still be attempted;
                    // its ACK is required, with no fresh deadline/retry.
                    #[cfg(test)]
                    self.reader.trace.lock().unwrap().push(if commit {
                        "commit_attempt"
                    } else {
                        "apply_rollback_attempt"
                    });
                    self.progress = out;
                    self.progress.completion = if commit {
                        Completion::CommitUnknown
                    } else {
                        Completion::RollbackUnknown
                    };
                    let cleanup_refusal = std::cell::Cell::new(None);
                    let control_check = || {
                        if commit {
                            check()
                        } else {
                            if let Err(e) = check() {
                                cleanup_refusal.set(Some(e));
                            }
                            if Instant::now() >= handle.candidate.deadline {
                                Err(ApplyError::Expired)
                            } else {
                                Ok(())
                            }
                        }
                    };
                    let result = quiet(tokio::time::timeout_at(
                        deadline,
                        guarded(
                            sqlx::query(if commit { "COMMIT" } else { "ROLLBACK" })
                                .execute(&mut *connection),
                            control_check,
                        ),
                    ))
                    .await
                    .map_err(|_| ApplyError::Expired)
                    .and_then(|r| r);
                    if let Some(e) = cleanup_refusal.get() {
                        out.refusal = Some(e);
                    }
                    let current = check();
                    out.completion = match (commit, matches!(result, Ok(Ok(_)))) {
                        (true, true) => Completion::Committed,
                        (true, false) => Completion::CommitUnknown,
                        (false, true) => Completion::RolledBack,
                        (false, false) => Completion::RollbackUnknown,
                    };
                    #[cfg(test)]
                    if matches!(result, Ok(Ok(_))) {
                        self.reader.trace.lock().unwrap().push(if commit {
                            "commit_ack"
                        } else {
                            "apply_rollback_ack"
                        });
                    }
                    // ACK records server completion even if authority expired
                    // afterwards. The separate refusal preserves failed validation.
                    if let Err(e) = current {
                        out.refusal = Some(e);
                    }
                } else if out.unacknowledged_step.is_none()
                    && out.rejected_step.is_none()
                    && out.acknowledged > 0
                {
                    out.completion = Completion::ImplicitCommitPrefix;
                }
                if let Err(e) = check() {
                    out.refusal = Some(e);
                }
                if out.completion == Completion::Committed && out.refusal.is_some() {
                    out.completion = Completion::CommitAcknowledgedCurrentInvalid;
                }
                self.progress = out;
                out
            }
        }
    };
}
#[cfg(feature = "mysql")]
pub use super::mysql::MysqlApply;
#[cfg(feature = "postgres")]
pub use super::postgres::PostgresApply;
pub(super) use apply_session;

#[cfg(test)]
mod tests;
