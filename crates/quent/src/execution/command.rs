//! Application-only durable identity and fixed private ledger protocol.
use super::*;
/// Durable ID, persisted by the app with its original input before submission.
/// Unlike an execution handle, this nominal ID can be reused in a new session.
#[derive(Clone, PartialEq, Eq)]
pub struct CommandId(pub(super) [u8; 16]);
impl CommandId {
    pub fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}
/// Trusted application namespace; no public wire input or authentication inference.
#[derive(Clone, PartialEq, Eq)]
pub struct CommandNamespace(pub(super) Vec<u8>);
impl CommandNamespace {
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > 128 {
            return Err(ExecutionError::Unsupported);
        }
        Ok(Self(bytes))
    }
}
/// App-installed ledger. Its table must share the business table's explicit
/// schema/catalog. Creation does not verify physical DDL/engine/permissions.
#[derive(Clone, PartialEq, Eq)]
pub struct CommandLedger(pub(super) Table);
impl CommandLedger {
    pub fn application_supplied(table: Table) -> Result<Self> {
        if table.sensitive_alias().is_some()
            || table.sensitive_path().len() != 2
            || table.sensitive_path().iter().any(|i| {
                !i.sensitive_text()
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            })
        {
            return Err(ExecutionError::Unsupported);
        }
        Ok(Self(table))
    }
}
/// Consumed at future creation. Never Clone/Copy/serde or returned after failure.
pub struct ValidatedCommand {
    pub(super) operation: ValidatedOperation,
    pub(super) key: CommandCanonical,
    pub(super) input: CommandCanonical,
    pub(super) ledger: CommandLedger,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandResultKind {
    FirstCommit,
    Replayed,
}
pub struct CommandResult {
    pub(super) result: ExecutionResult,
    pub(super) kind: CommandResultKind,
}
impl CommandResult {
    pub fn kind(&self) -> CommandResultKind {
        self.kind
    }
    pub fn sensitive_result(&self) -> &ExecutionResult {
        &self.result
    }
    pub fn public_view(&self) -> ResultView {
        self.result.public_view()
    }
}
macro_rules! redact {($($t:ty),*)=>{$(impl fmt::Debug for $t {fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($t),"([REDACTED])"))}})*};}
redact!(
    CommandId,
    CommandNamespace,
    CommandLedger,
    ValidatedCommand,
    CommandResult
);

pub(super) fn command_check(
    h: &ValidatedCommand,
    session: &Arc<()>,
    bound: &DatabaseContext,
    current: &DatabaseContext,
    op: &Operation,
) -> Result<()> {
    recheck(&h.operation, session, bound, current, Some(op))?;
    recheck_authorization(None, &h.operation, session)?;
    check_expiry(&h.operation, Instant::now())
}
// Untrusted storage is never expanded into IR or authority. Validate the bounded
// typed token grammar, then compare EVERY byte with newly prepared canonical input.
// Semantic shape/type changes, extra/duplicate fields and noncanonical encodings
// cannot compare equal. No digest, collation or partial/truncated comparison.
pub(super) fn inspect_input(bytes: &[u8], expected: &[u8]) -> Result<()> {
    fn tokens(mut b: &[u8], depth: usize, nodes: &mut usize) -> Result<()> {
        if depth > 32 {
            return Err(ExecutionError::LedgerInvalid);
        }
        while !b.is_empty() {
            *nodes += 1;
            if *nodes > 131072 || b.len() < 5 {
                return Err(ExecutionError::LedgerInvalid);
            }
            let tag = b[0];
            let n = u32::from_be_bytes(
                b[1..5]
                    .try_into()
                    .map_err(|_| ExecutionError::LedgerInvalid)?,
            ) as usize;
            let value = b
                .get(5..5usize.checked_add(n).ok_or(ExecutionError::LedgerInvalid)?)
                .ok_or(ExecutionError::LedgerInvalid)?;
            let valid = match tag {
                1 | 5 => n == 1,
                2 | 6 => n == 2,
                3 | 7 => n == 4,
                4 | 8 => n == 8,
                9 => n == 1 && value[0] <= 1,
                10 => n <= 4096 && std::str::from_utf8(value).is_ok(),
                11 => n <= 131072,
                12 => n == 1 && (1..=10).contains(&value[0]),
                128 => {
                    tokens(value, depth + 1, nodes)?;
                    true
                }
                _ => false,
            };
            if !valid {
                return Err(ExecutionError::LedgerInvalid);
            }
            b = &b[5 + n..];
        }
        Ok(())
    }
    if bytes.len() > 1_048_576 || bytes.get(..4) != Some(b"QCC1") {
        return Err(ExecutionError::LedgerInvalid);
    }
    tokens(&bytes[4..], 0, &mut 0)?;
    if bytes != expected {
        return Err(ExecutionError::IdentityConflict);
    }
    Ok(())
}
pub(super) fn saved_result(bytes: &[u8], dialect: Dialect) -> Result<ExecutionResult> {
    let tag = if dialect == Dialect::Postgres { 1 } else { 2 };
    if bytes.len() != 13 || &bytes[..4] != b"QCR1" || bytes[4] != tag {
        return Err(ExecutionError::LedgerInvalid);
    }
    let n = u64::from_be_bytes(
        bytes[5..]
            .try_into()
            .map_err(|_| ExecutionError::LedgerInvalid)?,
    );
    if n != 1 {
        return Err(ExecutionError::LedgerInvalid);
    }
    Ok(ExecutionResult {
        rows: vec![],
        count: None,
        affected: Some(if dialect == Dialect::Postgres {
            AffectedRows::PostgresMatched(n)
        } else {
            AffectedRows::MysqlFoundRows(n)
        }),
        state: ResultState::Committed,
    })
}
pub(super) fn result_bytes(result: &ExecutionResult, dialect: Dialect) -> Result<Vec<u8>> {
    if !result.rows.is_empty() || result.count.is_some() {
        return Err(ExecutionError::Unsupported);
    }
    let n = match (result.affected, dialect) {
        (Some(AffectedRows::PostgresMatched(n)), Dialect::Postgres)
        | (Some(AffectedRows::MysqlFoundRows(n)), Dialect::Mysql) => n,
        _ => return Err(ExecutionError::Unsupported),
    };
    if n != 1 {
        return Err(ExecutionError::ExpectationMismatch);
    }
    let mut bytes = b"QCR1".to_vec();
    bytes.push(if dialect == Dialect::Postgres { 1 } else { 2 });
    bytes.extend_from_slice(&n.to_be_bytes());
    Ok(bytes)
}

macro_rules! command_dispatch {
    ($driver:ident,$dialect:ident,$runner:ident) => {
        /// DB/provider-free, single INSERT/all-column or existing Low UPDATE.
        /// Caller owns correct full schema/transactional effects assertions.
        pub fn validate_command(
            &self,
            current: &DatabaseContext,
            operation: &Operation,
            namespace: &CommandNamespace,
            id: CommandId,
            ledger: &CommandLedger,
            ttl: Duration,
        ) -> Result<ValidatedCommand> {
            if !self.available {
                return Err(ExecutionError::SessionUnavailable);
            }
            let expires = deadline(Instant::now(), ttl)?;
            let prepared = current
                .prepare()
                .prepare_command_operation(operation)
                .map_err(ExecutionError::Policy)?;
            check_prepared(&self.context, current, &prepared, Dialect::$dialect)?;
            let source = prepared.sensitive_source();
            let final_source = source.final_source();
            let business = final_source.sensitive_table();
            if !current.presence().tenant
                || ledger.0 == *business
                || ledger.0.sensitive_path()[0] != business.sensitive_path()[0]
            {
                return Err(ExecutionError::Unsupported);
            }
            // The native capability check also runs before the first ledger send.
            let mut query = sqlx::query::<$driver>(sqlx::AssertSqlSafe(final_source.sql()));
            for v in final_source.args() {
                bind(&mut query, v)?;
            }
            drop(query);
            let key = source
                .canonical_command_key(&namespace.0, &id.0)
                .map_err(ExecutionError::Policy)?;
            let input = source
                .canonical_command_input(&ledger.0)
                .map_err(ExecutionError::Policy)?;
            Ok(ValidatedCommand {
                operation: ValidatedOperation {
                    prepared,
                    session: self.identity.clone(),
                    expires,
                },
                key,
                input,
                ledger: ledger.clone(),
            })
        }
        /// Consumes the handle even when this future is never polled. No retry.
        pub async fn execute_command(
            &mut self,
            current: &DatabaseContext,
            operation: &Operation,
            handle: ValidatedCommand,
        ) -> std::result::Result<CommandResult, AtomicFailure> {
            self.command_inner(current, operation, handle, false)
                .await?
                .ok_or(AtomicFailure {
                    cause: ExecutionError::Unsupported,
                    disposition: AtomicDisposition::NotStarted,
                })
        }
        /// Explicit durable-ID reconciliation in a newly validated current session.
        /// Missing is only a successful bounded read with no row; never DB/decode error.
        pub async fn lookup_command(
            &mut self,
            current: &DatabaseContext,
            operation: &Operation,
            handle: ValidatedCommand,
        ) -> std::result::Result<Option<CommandResult>, AtomicFailure> {
            self.command_inner(current, operation, handle, true).await
        }
        async fn command_inner(
            &mut self,
            current: &DatabaseContext,
            operation: &Operation,
            handle: ValidatedCommand,
            lookup_only: bool,
        ) -> std::result::Result<Option<CommandResult>, AtomicFailure> {
            quiet(async {
                let before = |cause| AtomicFailure {
                    cause,
                    disposition: AtomicDisposition::NotStarted,
                };
                if !self.available {
                    return Err(before(ExecutionError::SessionUnavailable));
                }
                command_check(&handle, &self.identity, &self.context, current, operation)
                    .map_err(before)?;
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
                let mut runner =
                    $runner::internal(tx.as_mut(), self.context.clone(), self.identity.clone());
                #[cfg(test)]
                {
                    runner.recorder.live = self.recorder.live.clone();
                    runner.final_now = self.final_now.take();
                    runner.final_barrier = self.final_barrier.take();
                }
                let checked = runner
                    .command_protocol(current, operation, &handle, lookup_only)
                    .await;
                #[cfg(test)]
                {
                    self.recorder.calls.append(&mut runner.recorder.calls);
                    self.recorder
                        .sql_addresses
                        .append(&mut runner.recorder.sql_addresses);
                    self.recorder.metadata.append(&mut runner.recorder.metadata);
                    self.recorder.events.append(&mut runner.recorder.events);
                }
                drop(runner);
                let checked = checked.and_then(|result| {
                    command_check(&handle, &self.identity, &self.context, current, operation)?;
                    Ok(result)
                });
                match checked {
                    Ok(result) => {
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
// Protocol is available only on private internal owned-Tx runners. There is no
// public borrowed command method, raw executor or caller-supplied protocol SQL.
macro_rules! command_protocol {
    ($driver:ident,$dialect:ident) => {
        async fn command_protocol(&mut self,current:&DatabaseContext,operation:&Operation,h:&ValidatedCommand,lookup_only:bool)->Result<Option<CommandResult>> {
            let table=h.ledger.0.sensitive_qualified_sql(Dialect::$dialect);
            let (p1,p2)=if Dialect::$dialect==Dialect::Postgres{("$1","$2")}else{("?","?")};
            let select=format!("SELECT version, SUBSTRING(input,1,1048577), SUBSTRING(result,1,14) FROM {table} WHERE identity = {p1} LIMIT 2");
            let query=sqlx::query::<$driver>(sqlx::AssertSqlSafe(select)).bind(h.key.sensitive_bytes());
            command_check(h,&self.identity,&self.context,current,operation)?;
            #[cfg(test)] self.recorder.event("ledger_lookup");
            let mut rows=query.fetch_all(self.connection.native()).await.map_err(|_|ExecutionError::LedgerFailed)?;
            command_check(h,&self.identity,&self.context,current,operation)?;
            if rows.len()>1 {return Err(ExecutionError::LedgerInvalid);}
            if let Some(row)=rows.pop() {
                let version:i16=row.try_get(0).map_err(|_|ExecutionError::LedgerInvalid)?;
                let input:Vec<u8>=row.try_get(1).map_err(|_|ExecutionError::LedgerInvalid)?;
                let result:Option<Vec<u8>>=row.try_get(2).map_err(|_|ExecutionError::LedgerInvalid)?;
                if version!=1 {return Err(ExecutionError::LedgerInvalid);}
                inspect_input(&input,h.input.sensitive_bytes())?;
                let result=saved_result(result.as_deref().ok_or(ExecutionError::ReservationUnresolved)?,Dialect::$dialect)?;
                command_check(h,&self.identity,&self.context,current,operation)?;
                return Ok(Some(CommandResult{result,kind:CommandResultKind::Replayed}));
            }
            if lookup_only {return Ok(None);}
            let insert=format!("INSERT INTO {table} (identity,input,result,version) VALUES ({p1},{p2},NULL,1)");
            let query=sqlx::query::<$driver>(sqlx::AssertSqlSafe(insert)).bind(h.key.sensitive_bytes()).bind(h.input.sensitive_bytes());
            command_check(h,&self.identity,&self.context,current,operation)?;
            #[cfg(test)] self.recorder.event("ledger_reserve");
            let reserved=query.execute(self.connection.native()).await.map_err(|_|ExecutionError::LedgerFailed)?;
            command_check(h,&self.identity,&self.context,current,operation)?;
            if reserved.rows_affected()!=1 {return Err(ExecutionError::LedgerInvalid);}
            #[cfg(test)] self.recorder.event("command_ready");
            let mut result=self.dispatch_retained(current,Some(operation),&h.operation,None).await?;
            command_check(h,&self.identity,&self.context,current,operation)?;
            let bytes=result_bytes(&result,Dialect::$dialect)?;
            let update=format!("UPDATE {table} SET result = {p1} WHERE identity = {p2} AND result IS NULL AND version = 1");
            let query=sqlx::query::<$driver>(sqlx::AssertSqlSafe(update)).bind(bytes).bind(h.key.sensitive_bytes());
            command_check(h,&self.identity,&self.context,current,operation)?;
            #[cfg(test)] self.recorder.event("ledger_result");
            let saved=query.execute(self.connection.native()).await.map_err(|_|ExecutionError::LedgerFailed)?;
            command_check(h,&self.identity,&self.context,current,operation)?;
            if saved.rows_affected()!=1 {return Err(ExecutionError::LedgerInvalid);}
            result.state=ResultState::Committed;
            Ok(Some(CommandResult{result,kind:CommandResultKind::FirstCommit}))
        }
    }
}
