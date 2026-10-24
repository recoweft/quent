//! One gate and a closed adapter set. No raw SQL/Plan/source execution input.
use quent_core::*;
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
    operation: &Operation,
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
        .same_bounded_input(operation)
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
}
impl fmt::Debug for ExecutionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExecutionResult([REDACTED])")
    }
}
impl ExecutionResult {
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
    Ok(p)
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

// Finite test-only observations at the real final dispatch point, after all
// preflight checks/encoding. No mock executor, callback or public recorder API.
#[cfg(test)]
#[derive(Default)]
struct Recorder {
    calls: Vec<(String, Vec<Value>)>,
}
#[cfg(test)]
impl Recorder {
    fn sent(&mut self, sql: &str, args: &[Value]) {
        assert!(
            self.calls.len() < 128,
            "finite dispatcher observation budget"
        );
        self.calls.push((sql.to_owned(), args.to_vec()));
    }
}
// A closed macro expands the identical immediate/delayed dispatcher for the two owned
// driver connections. Binding/decoding are finite private adapter functions; no
// caller can implement or supply a backend, binder, executor or callback.
macro_rules! immediate_dispatch {
    ($driver:ident, $dialect:ident, $affected:ident) => {
        #[cfg(test)]
        pub(super) async fn test_connection_id(&mut self) -> i64 {
            // Explicit test observation setup, never a business dispatcher call.
            let sql = if Dialect::$dialect == Dialect::Postgres {
                "SELECT CAST(pg_backend_pid() AS BIGINT)"
            } else {
                "SELECT CAST(CONNECTION_ID() AS SIGNED)"
            };
            sqlx::query::<$driver>(sqlx::AssertSqlSafe(sql.to_owned()))
                .fetch_one(&mut self.connection)
                .await
                .unwrap()
                .get(0)
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
            let result = if matches!(family, Family::Update | Family::Delete) {
                query
                    .execute(&mut self.connection)
                    .await
                    .map(|r| ExecutionResult {
                        rows: vec![],
                        count: None,
                        affected: Some(AffectedRows::$affected(r.rows_affected())),
                    })
                    .map_err(|_| ExecutionError::OutcomeUnknown)
            } else if matches!(family, Family::Select | Family::Count) {
                let mut stream = query.fetch(&mut self.connection);
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
                            Ok(r) => rows.push(r),
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
                    Ok(ExecutionResult {
                        rows,
                        count,
                        affected: None,
                    })
                }
            } else {
                Err(ExecutionError::Unsupported)
            };
            if result.is_ok() {
                self.available = true;
            }
            result
        }
    };
}

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::PostgresSession;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "mysql")]
pub use mysql::MysqlSession;
#[cfg(test)]
mod tests;
