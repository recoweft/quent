//! One gate and a closed adapter set. No raw SQL/Plan/source execution input.
// Never forward SQLx protocol notices/type-resolution fields to the application's
// tracing subscriber. Statement/slow logging stays disabled independently.
pub(crate) mod quiet_driver;
use quent_core::*;
use quiet_driver::quiet;
use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};

/// Fixed, redacted classifications; no SQLx error or source chain is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionError {
    Policy(PolicyError),
    ContextMismatch,
    OperationMismatch,
    InvalidTtl,
    Expired,
    MissingSubject,
    RiskRefused,
    Incomplete,
    Unsupported,
    Encode,
    Connect,
    ReadFailed,
    Decode,
    ResultLimit,
    OutcomeUnknown,
    SessionUnavailable,
    ExpectationMismatch,
    Conflict,
}
impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_EXECUTION_{self:?}")
    }
}
impl std::error::Error for ExecutionError {}
type Result<T> = std::result::Result<T, ExecutionError>;

/// A consumed structural validation result, never business authorization or a permit.
/// No public source/getter/restoration path exists. A new attempt needs validation.
/// ```compile_fail
/// fn duplicate(h: quent::ValidatedOperation) { let _ = h.clone(); }
/// ```
/// ```compile_fail
/// fn copy<T: Copy>() {} copy::<quent::ValidatedOperation>();
/// ```
/// ```compile_fail
/// fn export(h: &quent::ValidatedOperation) { serde_json::to_string(h).unwrap(); }
/// ```
/// ```compile_fail
/// let _: quent::ValidatedOperation = serde_json::from_str("{}").unwrap();
/// ```
/// ```compile_fail
/// fn consume(_: quent::ValidatedOperation) {}
/// fn again(h: quent::ValidatedOperation) { consume(h); consume(h); }
/// ```
pub struct ValidatedOperation {
    prepared: PolicyPreparation,
    session: Arc<()>,
    expires: Instant,
}
impl fmt::Debug for ValidatedOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ValidatedOperation([REDACTED])")
    }
}
const MAX_TTL: Duration = Duration::from_secs(300);
fn deadline(now: Instant, ttl: Duration) -> Result<Instant> {
    if ttl.is_zero() || ttl > MAX_TTL {
        return Err(ExecutionError::InvalidTtl);
    }
    now.checked_add(ttl).ok_or(ExecutionError::InvalidTtl)
}
fn recheck(
    handle: &ValidatedOperation,
    session: &Arc<()>,
    bound: &DatabaseContext,
    current: &DatabaseContext,
    operation: Option<&Operation>,
) -> Result<()> {
    let source = handle.prepared.sensitive_source();
    if !Arc::ptr_eq(&handle.session, session)
        || !bound.same_snapshot(current)
        || !source.context().same_snapshot(current)
    {
        return Err(ExecutionError::ContextMismatch);
    }
    if !source
        .original_operation()
        .same_bounded_input(operation.unwrap_or_else(|| source.original_operation()))
        .map_err(|_| ExecutionError::OperationMismatch)?
    {
        return Err(ExecutionError::OperationMismatch);
    }
    Ok(())
}
fn check_expiry(handle: &ValidatedOperation, now: Instant) -> Result<()> {
    if now >= handle.expires {
        Err(ExecutionError::Expired)
    } else {
        Ok(())
    }
}

/// A bounded exact expectation; never inferred from the first returned row.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ExpectedRows(u16);
impl ExpectedRows {
    pub fn exactly(rows: u16) -> Result<Self> {
        if usize::from(rows) > MAX_ROWS {
            return Err(ExecutionError::Unsupported);
        }
        Ok(Self(rows))
    }
}
impl fmt::Debug for ExpectedRows {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExpectedRows([REDACTED])")
    }
}
/// Consumed binding of the entire operation, current snapshot and expectation.
pub struct ValidatedAtomicOperation {
    operation: ValidatedOperation,
    expected: ExpectedRows,
}
impl fmt::Debug for ValidatedAtomicOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ValidatedAtomicOperation([REDACTED])")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultState {
    Observed,
    Uncommitted,
    Committed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicDisposition {
    NotStarted,
    RolledBack,
    OutcomeUnknown,
}
/// No driver source, SQL, arbitrary reason or sensitive result is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtomicFailure {
    cause: ExecutionError,
    disposition: AtomicDisposition,
}
impl AtomicFailure {
    pub fn cause(&self) -> ExecutionError {
        self.cause
    }
    pub fn disposition(&self) -> AtomicDisposition {
        self.disposition
    }
}
impl fmt::Display for AtomicFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_ATOMIC_{:?}_{:?}", self.disposition, self.cause)
    }
}
impl std::error::Error for AtomicFailure {}
// Private connection access, implemented only by the two native connections.
trait NativeConnection {
    type Connection;
    fn native(&mut self) -> &mut Self::Connection;
}
impl<C: NativeConnection> NativeConnection for &mut C {
    type Connection = C::Connection;
    fn native(&mut self) -> &mut Self::Connection {
        (**self).native()
    }
}

/// Affected rows are observations, never an exactly-one or atomicity guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffectedRows {
    PostgresMatched(u64),
    MysqlFoundRows(u64),
}

/// Values are available only by explicit sensitive access; ordinary Debug is fixed.
pub struct ExecutionResult {
    rows: Vec<Vec<Value>>,
    count: Option<Value>,
    affected: Option<AffectedRows>,
    state: ResultState,
}
impl fmt::Debug for ExecutionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExecutionResult([REDACTED])")
    }
}
impl ExecutionResult {
    pub fn state(&self) -> ResultState {
        self.state
    }
    pub fn sensitive_rows(&self) -> &[Vec<Value>] {
        &self.rows
    }
    pub fn sensitive_count(&self) -> Option<&Value> {
        self.count.as_ref()
    }
    pub fn affected_rows(&self) -> Option<AffectedRows> {
        self.affected
    }
}
const MAX_ROWS: usize = 256;
const MAX_TEXT_BYTES: usize = 4096;
const MAX_RESULT_BYTES: usize = 1_048_576;

// The exact immutable operation is prepared here, then this very preparation is
// borrowed by the selected adapter. No public API accepts a preparation or source.
fn prepare(
    bound: &DatabaseContext,
    current: &DatabaseContext,
    operation: &Operation,
    dialect: Dialect,
) -> Result<PolicyPreparation> {
    if !bound.same_snapshot(current) || current.sensitive_source().target().3 != dialect {
        return Err(ExecutionError::ContextMismatch);
    }
    if current.sensitive_source().subject().is_none() {
        return Err(ExecutionError::MissingSubject);
    }
    let p = current
        .prepare()
        .prepare_operation(operation)
        .map_err(ExecutionError::Policy)?;
    check_prepared(bound, current, &p, dialect)?;
    Ok(p)
}
// Shared by ordinary validation and compiled reads; a typed verdict never skips it.
fn check_prepared(
    bound: &DatabaseContext,
    current: &DatabaseContext,
    p: &PolicyPreparation,
    dialect: Dialect,
) -> Result<()> {
    if !bound.same_snapshot(current)
        || !p.sensitive_source().context().same_snapshot(current)
        || current.sensitive_source().target().3 != dialect
    {
        return Err(ExecutionError::ContextMismatch);
    }
    if current.sensitive_source().subject().is_none() {
        return Err(ExecutionError::MissingSubject);
    }
    let inspection = p.inspect();
    if matches!(
        inspection.risk(),
        Risk::High | Risk::Destructive | Risk::Blocked
    ) {
        return Err(ExecutionError::RiskRefused);
    }
    if inspection.completeness() != Completeness::Complete
        || matches!(inspection.scope(), Scope::Unknown | Scope::NotApplicable)
    {
        return Err(ExecutionError::Incomplete);
    }
    Ok(())
}
fn check_result_size(row: &[Value], used: &mut usize) -> Result<()> {
    for v in row {
        let n = match v {
            Value::Text(s) => {
                if s.len() > MAX_TEXT_BYTES {
                    return Err(ExecutionError::ResultLimit);
                }
                s.len()
            }
            _ => 8,
        };
        *used = used.checked_add(n).ok_or(ExecutionError::ResultLimit)?;
        if *used > MAX_RESULT_BYTES {
            return Err(ExecutionError::ResultLimit);
        }
    }
    Ok(())
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum AtomicFault {
    Begin,
    Commit,
    CommitReplyLost,
    Rollback,
    RollbackReplyLost,
}

// Finite test-only observations at the real final dispatch point, after all
// preflight checks/encoding. No mock executor, callback or public recorder API.
#[cfg(test)]
#[derive(Default)]
struct Recorder {
    calls: Vec<(String, Vec<Value>)>,
    sql_addresses: Vec<usize>,
    events: Vec<&'static str>,
    live: Arc<std::sync::Mutex<Vec<&'static str>>>,
}
#[cfg(test)]
impl Recorder {
    fn event(&mut self, event: &'static str) {
        assert!(
            self.events.len() < 65_536,
            "finite event observation budget"
        );
        self.events.push(event);
        let mut live = self.live.lock().unwrap();
        assert!(live.len() < 65_536, "finite cancellation trace budget");
        live.push(event);
    }
    fn sent(&mut self, sql: &str, args: &[Value]) {
        assert!(
            self.calls.len() < 128,
            "finite dispatcher observation budget"
        );
        self.event("send");
        self.sql_addresses.push(sql.as_ptr() as usize);
        self.calls.push((sql.to_owned(), args.to_vec()));
    }
}
// A closed macro expands the identical immediate/delayed dispatcher for the two owned
// driver connections. Binding/decoding are finite private adapter functions; no
// caller can implement or supply a backend, binder, executor or callback.
macro_rules! immediate_dispatch {
    ($driver:ident, $dialect:ident, $affected:ident) => {
        #[cfg(test)]
        #[allow(dead_code)]
        pub(super) async fn test_connection_id(&mut self) -> i64 {
            // Explicit test observation setup, never a business dispatcher call.
            let sql = if Dialect::$dialect == Dialect::Postgres {
                "SELECT CAST(pg_backend_pid() AS BIGINT)"
            } else {
                "SELECT CAST(CONNECTION_ID() AS SIGNED)"
            };
            sqlx::query::<$driver>(sqlx::AssertSqlSafe(sql.to_owned()))
                .fetch_one(self.connection.native())
                .await
                .unwrap()
                .get(0)
        }
        /// A quarantined session/adapter cannot be reused after a canceled submission.
        pub fn is_available(&self) -> bool {
            self.available
        }
        /// DB/provider-free validation bound to this exact connection lifetime.
        /// TTL must be greater than zero and at most 300 seconds, exclusive expiry.
        pub fn validate(
            &self,
            current: &DatabaseContext,
            operation: &Operation,
            ttl: Duration,
        ) -> Result<ValidatedOperation> {
            if !self.available {
                return Err(ExecutionError::SessionUnavailable);
            }
            let expires = deadline(Instant::now(), ttl)?;
            let prepared = prepare(&self.context, current, operation, Dialect::$dialect)?;
            Ok(ValidatedOperation {
                prepared,
                session: self.identity.clone(),
                expires,
            })
        }
        /// Immediate execution uses the identical validation and final dispatch gate.
        pub async fn execute(
            &mut self,
            current: &DatabaseContext,
            operation: Operation,
        ) -> Result<ExecutionResult> {
            let handle = self.validate(current, &operation, MAX_TTL)?;
            self.execute_validated(current, &operation, handle).await
        }
        /// Takes ownership when the future is constructed, including if never polled.
        /// Failure/cancellation never returns the handle and never retries a query.
        pub async fn execute_validated(
            &mut self,
            current: &DatabaseContext,
            operation: &Operation,
            handle: ValidatedOperation,
        ) -> Result<ExecutionResult> {
            self.dispatch_prepared(current, Some(operation), handle)
                .await
        }
        async fn dispatch_prepared(
            &mut self,
            current: &DatabaseContext,
            operation: Option<&Operation>,
            handle: ValidatedOperation,
        ) -> Result<ExecutionResult> {
            quiet(async {
                if !self.available {
                    return Err(ExecutionError::SessionUnavailable);
                }
                recheck(&handle, &self.identity, &self.context, current, operation)?;
                check_expiry(&handle, Instant::now())?;
                let prepared = &handle.prepared;
                let source = prepared.sensitive_source();
                let record = source.final_source();
                let family = record.metadata().family();
                let columns = record.result_columns();
                if !matches!(
                    family,
                    Family::Select | Family::Count | Family::Update | Family::Delete
                ) {
                    return Err(ExecutionError::Unsupported);
                }
                // SQLx 0.9 requires the safe AssertSqlSafe marker for dynamic SQL.
                // Only the sole Core renderer produced this exact immutable SQL; identifiers
                // were validated/quoted and all values remain native bound arguments.
                // Finite capability and encoding matrix, completed before any query send.
                let mut query = sqlx::query::<$driver>(sqlx::AssertSqlSafe(record.sql()));
                for value in record.args() {
                    bind(&mut query, value)?;
                }
                // No application await occurs between this exact check and the SQLx send.
                // &mut self and immutable current/operation borrows exclude app substitution.
                #[cfg(test)]
                if let Some(barrier) = self.final_barrier.take() {
                    barrier.notified().await;
                }
                recheck(&handle, &self.identity, &self.context, current, operation)?;
                // Sample the clock AFTER bounded input/context comparison, never before it.
                #[cfg(test)]
                let now = self.final_now.take().unwrap_or_else(Instant::now);
                #[cfg(not(test))]
                let now = Instant::now();
                check_expiry(&handle, now)?;
                self.available = false; // cancellation/error leaves this session unusable
                #[cfg(test)]
                self.recorder.sent(record.sql(), record.args());
                let state = self.result_state;
                let result = if matches!(family, Family::Update | Family::Delete)
                    && columns.is_empty()
                {
                    query
                        .execute(self.connection.native())
                        .await
                        .map(|r| ExecutionResult {
                            state,
                            rows: vec![],
                            count: None,
                            affected: Some(AffectedRows::$affected(r.rows_affected())),
                        })
                        .map_err(|_| ExecutionError::OutcomeUnknown)
                } else if matches!(family, Family::Select | Family::Count) || !columns.is_empty() {
                    let mut stream = query.fetch(self.connection.native());
                    let mut rows = Vec::new();
                    let mut count = None;
                    let mut used = 0;
                    let mut failure = None;
                    loop {
                        let row = match stream.try_next().await {
                            Ok(Some(row)) => row,
                            Ok(None) => break,
                            Err(_) => {
                                failure = Some(ExecutionError::ReadFailed);
                                break;
                            }
                        };
                        if family == Family::Count {
                            if count.is_some() || row.len() != 1 {
                                failure = Some(ExecutionError::Decode);
                                break;
                            }
                            match decode_count(&row) {
                                Ok(v) => count = Some(v),
                                Err(e) => {
                                    failure = Some(e);
                                    break;
                                }
                            }
                        } else {
                            if rows.len() == MAX_ROWS {
                                failure = Some(ExecutionError::ResultLimit);
                                break;
                            }
                            match decode_row(&row, &columns).and_then(|r| {
                                check_result_size(&r, &mut used)?;
                                Ok(r)
                            }) {
                                Ok(r) => {
                                    #[cfg(test)]
                                    self.recorder.event("row_decoded");
                                    rows.push(r)
                                }
                                Err(e) => {
                                    failure = Some(e);
                                    break;
                                }
                            }
                        }
                    }
                    if let Some(e) = failure {
                        Err(e)
                    } else if family == Family::Count && count.is_none() {
                        Err(ExecutionError::Decode)
                    } else {
                        let affected = if matches!(family, Family::Update | Family::Delete) {
                            Some(AffectedRows::$affected(rows.len() as u64))
                        } else {
                            None
                        };
                        Ok(ExecutionResult {
                            state,
                            rows,
                            count,
                            affected,
                        })
                    }
                } else {
                    Err(ExecutionError::Unsupported)
                };
                #[cfg(test)]
                self.recorder.event(if result.is_ok() {
                    "decoded"
                } else {
                    "decode_or_driver_error"
                });
                if result.is_ok() {
                    self.available = true;
                }
                result
            })
            .await
        }
    };
}

macro_rules! compiled_dispatch {
    ($dialect:ident) => {
        /// Fail closed if a generated finite result decoder rejects native data.
        /// This only revokes session availability; it grants no execution access.
        #[cfg(feature = "codegen")]
        pub fn quarantine_generated_decode(&mut self) {
            self.available = false;
        }
        /// Application-only explicit read, using the compile-time sealed record.
        /// Current mapping/context are borrowed for the complete dispatch future.
        #[cfg(feature = "operation-spec")]
        pub async fn execute_compiled_read(
            &mut self,
            current: &DatabaseContext,
            mapping: &crate::operation_spec::TypedModelContract,
            compiled: crate::operation_spec::CompiledRead,
        ) -> Result<ExecutionResult> {
            if !self.available {
                return Err(ExecutionError::SessionUnavailable);
            }
            let expires = deadline(Instant::now(), MAX_TTL)?;
            let prepared = compiled
                .into_prepared(current, mapping)
                .map_err(|_| ExecutionError::ContextMismatch)?;
            check_prepared(&self.context, current, &prepared, Dialect::$dialect)?;
            let handle = ValidatedOperation {
                prepared,
                session: self.identity.clone(),
                expires,
            };
            self.dispatch_prepared(current, None, handle).await
        }
    };
}

// A finite owned transaction is one checked business operation. No public commit,
// callback, raw executor, savepoint or partially validated transaction is exposed.
macro_rules! atomic_dispatch {
    ($driver:ident, $dialect:ident, $runner:ident) => {
        pub fn validate_atomic(
            &self,
            current: &DatabaseContext,
            operation: &Operation,
            expected: ExpectedRows,
            ttl: Duration,
        ) -> Result<ValidatedAtomicOperation> {
            let operation = self.validate(current, operation, ttl)?;
            let source = operation.prepared.sensitive_source();
            if !matches!(
                source.final_source().metadata().family(),
                Family::Update | Family::Delete
            ) || (source.final_source().metadata().optimistic_version() && expected.0 != 1)
            {
                return Err(ExecutionError::Unsupported);
            }
            Ok(ValidatedAtomicOperation {
                operation,
                expected,
            })
        }
        /// Own begin, exactly one gated business operation, full consumption,
        /// expectation comparison, and commit or explicit rollback, in that order.
        /// Cancel/drop returns no result and permanently quarantines this session.
        pub async fn execute_atomic(
            &mut self,
            current: &DatabaseContext,
            operation: &Operation,
            expected: ExpectedRows,
            handle: ValidatedAtomicOperation,
        ) -> std::result::Result<ExecutionResult, AtomicFailure> {
            quiet(async {
                let before = |cause| AtomicFailure {
                    cause,
                    disposition: AtomicDisposition::NotStarted,
                };
                if !self.available {
                    return Err(before(ExecutionError::SessionUnavailable));
                }
                if handle.expected != expected {
                    return Err(before(ExecutionError::OperationMismatch));
                }
                recheck(
                    &handle.operation,
                    &self.identity,
                    &self.context,
                    current,
                    Some(operation),
                )
                .map_err(before)?;
                check_expiry(&handle.operation, Instant::now()).map_err(before)?;
                let version = handle
                    .operation
                    .prepared
                    .sensitive_source()
                    .final_source()
                    .metadata()
                    .optimistic_version();
                self.available = false;
                #[cfg(test)]
                self.recorder.event("begin");
                #[cfg(test)]
                let fault = self.atomic_fault.take();
                #[cfg(test)]
                let tx = if fault == Some(AtomicFault::Begin) {
                    Err(sqlx::Error::PoolClosed)
                } else {
                    self.connection.begin().await
                };
                #[cfg(not(test))]
                let tx = self.connection.begin().await;
                let mut tx = tx.map_err(|_| AtomicFailure {
                    cause: ExecutionError::OutcomeUnknown,
                    disposition: AtomicDisposition::OutcomeUnknown,
                })?;
                #[cfg(test)]
                self.recorder.event("begun");
                // Transaction holds the exact connection borrow. It has no independently
                // exposed validation/executor identity. Recheck includes the begin await.
                let mut runner =
                    $runner::internal(tx.as_mut(), self.context.clone(), self.identity.clone());
                #[cfg(test)]
                {
                    runner.recorder.live = self.recorder.live.clone();
                    runner.final_now = self.final_now.take();
                    runner.final_barrier = self.final_barrier.take();
                }
                let result = runner
                    .execute_validated(current, operation, handle.operation)
                    .await;
                #[cfg(test)]
                {
                    self.recorder.calls.append(&mut runner.recorder.calls);
                    self.recorder.events.append(&mut runner.recorder.events);
                }
                drop(runner);
                let checked = result.and_then(|result| {
                    let actual = match result.affected {
                        Some(
                            AffectedRows::PostgresMatched(n) | AffectedRows::MysqlFoundRows(n),
                        ) => n,
                        None => return Err(ExecutionError::Unsupported),
                    };
                    if actual != u64::from(expected.0) {
                        return Err(if version && actual == 0 {
                            ExecutionError::Conflict
                        } else {
                            ExecutionError::ExpectationMismatch
                        });
                    }
                    Ok(result)
                });
                #[cfg(test)]
                self.recorder.event("expectation");
                match checked {
                    Ok(mut result) => {
                        #[cfg(test)]
                        self.recorder.event("commit");
                        #[cfg(test)]
                        let completion = match fault {
                            Some(AtomicFault::Commit) => {
                                drop(tx);
                                Err(sqlx::Error::PoolClosed)
                            }
                            Some(AtomicFault::CommitReplyLost) => {
                                tx.commit().await.and(Err(sqlx::Error::PoolClosed))
                            }
                            _ => tx.commit().await,
                        };
                        #[cfg(not(test))]
                        let completion = tx.commit().await;
                        completion.map_err(|_| AtomicFailure {
                            cause: ExecutionError::OutcomeUnknown,
                            disposition: AtomicDisposition::OutcomeUnknown,
                        })?;
                        #[cfg(test)]
                        self.recorder.event("committed");
                        self.available = true;
                        result.state = ResultState::Committed;
                        Ok(result)
                    }
                    Err(cause) => {
                        #[cfg(test)]
                        self.recorder.event("rollback");
                        #[cfg(test)]
                        let rollback = match fault {
                            Some(AtomicFault::Rollback) => {
                                drop(tx);
                                Err(sqlx::Error::PoolClosed)
                            }
                            Some(AtomicFault::RollbackReplyLost) => {
                                tx.rollback().await.and(Err(sqlx::Error::PoolClosed))
                            }
                            _ => tx.rollback().await,
                        };
                        #[cfg(not(test))]
                        let rollback = tx.rollback().await;
                        let disposition = if rollback.is_ok() {
                            #[cfg(test)]
                            self.recorder.event("rolled_back");
                            self.available = true;
                            AtomicDisposition::RolledBack
                        } else {
                            AtomicDisposition::OutcomeUnknown
                        };
                        Err(AtomicFailure { cause, disposition })
                    }
                }
            })
            .await
        }
    };
}
macro_rules! borrowed_dispatch {
    ($name:ident, $driver:ident, $connection:ident, $dialect:ident, $affected:ident) => {
        /// Exclusive borrowed transaction adapter. The caller can commit after this
        /// adapter is dropped, even after an error. Context/connection correspondence
        /// is an application assertion. Quent never commits or rolls it back.
        pub struct $name<'a> {
            connection: &'a mut $connection,
            context: DatabaseContext,
            available: bool,
            identity: Arc<()>,
            result_state: ResultState,
            #[cfg(test)]
            final_now: Option<Instant>,
            #[cfg(test)]
            final_barrier: Option<Arc<tokio::sync::Notify>>,
            #[cfg(test)]
            pub(super) recorder: Recorder,
        }
        impl fmt::Debug for $name<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "([REDACTED])"))
            }
        }
        impl<'a> $name<'a> {
            pub fn new(
                transaction: &'a mut sqlx::Transaction<'_, $driver>,
                context: DatabaseContext,
            ) -> Result<Self> {
                if context.sensitive_source().target().3 != Dialect::$dialect {
                    return Err(ExecutionError::ContextMismatch);
                }
                Ok(Self::internal(transaction.as_mut(), context, Arc::new(())))
            }
            fn internal(
                connection: &'a mut $connection,
                context: DatabaseContext,
                identity: Arc<()>,
            ) -> Self {
                Self {
                    connection,
                    context,
                    available: true,
                    identity,
                    result_state: ResultState::Uncommitted,
                    #[cfg(test)]
                    final_now: None,
                    #[cfg(test)]
                    final_barrier: None,
                    #[cfg(test)]
                    recorder: Recorder::default(),
                }
            }
            pub fn validate_atomic(
                &self,
                _: &DatabaseContext,
                _: &Operation,
                _: ExpectedRows,
                _: Duration,
            ) -> Result<ValidatedAtomicOperation> {
                Err(ExecutionError::Unsupported)
            }
            immediate_dispatch!($driver, $dialect, $affected);
        }
    };
}

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::{BorrowedPostgresTransaction, PostgresSession};
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "mysql")]
pub use mysql::{BorrowedMysqlTransaction, MysqlSession};
#[cfg(test)]
mod tests;

impl ExecutionError {
    pub fn public_view(&self) -> DiagnosticView {
        let code = match self {
            Self::Policy(e) => return e.public_view(),
            Self::ContextMismatch => DiagnosticCode::ExecutionContextMismatch,
            Self::OperationMismatch => DiagnosticCode::ExecutionOperationMismatch,
            Self::InvalidTtl => DiagnosticCode::ExecutionInvalidTtl,
            Self::Expired => DiagnosticCode::ExecutionExpired,
            Self::MissingSubject => DiagnosticCode::ExecutionMissingSubject,
            Self::RiskRefused => DiagnosticCode::ExecutionRiskRefused,
            Self::Incomplete => DiagnosticCode::ExecutionIncomplete,
            Self::Unsupported => DiagnosticCode::ExecutionUnsupported,
            Self::Encode => DiagnosticCode::ExecutionEncode,
            Self::Connect => DiagnosticCode::ExecutionConnect,
            Self::ReadFailed => DiagnosticCode::ExecutionReadFailed,
            Self::Decode => DiagnosticCode::ExecutionDecode,
            Self::ResultLimit => DiagnosticCode::ExecutionResultLimit,
            Self::OutcomeUnknown => DiagnosticCode::ExecutionOutcomeUnknown,
            Self::SessionUnavailable => DiagnosticCode::ExecutionSessionUnavailable,
            Self::ExpectationMismatch => DiagnosticCode::ExecutionExpectationMismatch,
            Self::Conflict => DiagnosticCode::ExecutionConflict,
        };
        DiagnosticView { code, state: None }
    }
}
impl AtomicFailure {
    pub fn public_view(&self) -> DiagnosticView {
        let mut v = self.cause.public_view();
        v.state = Some(match self.disposition {
            AtomicDisposition::NotStarted => ViewState::NotStarted,
            AtomicDisposition::RolledBack => ViewState::RolledBack,
            AtomicDisposition::OutcomeUnknown => ViewState::OutcomeUnknown,
        });
        v
    }
}
/// Output-only result state; contains no SQL result, COUNT value or affected rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultView {
    pub state: ViewState,
}
impl ResultView {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"kind\":\"quent_result_view\",\"version\":1,\"state\":\"{}\"}}",
            self.state.code()
        )
    }
}
impl fmt::Display for ResultView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
impl ExecutionResult {
    pub fn public_view(&self) -> ResultView {
        ResultView {
            state: match self.state {
                ResultState::Observed => ViewState::Observed,
                ResultState::Uncommitted => ViewState::Uncommitted,
                ResultState::Committed => ViewState::Committed,
            },
        }
    }
}
