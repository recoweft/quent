//! Application-only finite migration observations; never migration authorization.
use super::*;
use quent_core::migration::{
    MigrationDefault, MigrationPhase, MigrationPlan, MigrationStatus, Observed, PhaseCapability,
    SchemaAuthority,
};
use quent_core::{SqlType, Value};

/// Caller assertion about external DDL exclusion, not a lock verified by Quent.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DdlExclusion {
    Uncontrolled,
    ApplicationDeclared(std::num::NonZeroU64),
}
redacted!(DdlExclusion);
/// Exact role and caller-managed visibility generation. Catalog visibility is
/// additionally checked by the finite adapter. This does not authenticate a role.
#[derive(Clone, PartialEq, Eq)]
pub struct Visibility {
    pub(super) role: String,
    pub(super) generation: std::num::NonZeroU64,
}
impl Visibility {
    pub fn new(role: &str, generation: std::num::NonZeroU64) -> Result<Self> {
        if role.is_empty() || role.len() > 128 || role.chars().any(char::is_control) {
            return Err(VerifyError::Invalid);
        }
        Ok(Self {
            role: role.into(),
            generation,
        })
    }
}
redacted!(Visibility);
/// Typed private request. Neither public JSON nor a manifest can construct this.
/// The exact immutable plan owns phase/step/condition ordering.
pub struct PreflightRequest<'p> {
    pub(super) plan: &'p MigrationPlan,
    pub(super) phase: MigrationPhase,
    pub(super) scope: Scope,
    pub(super) visibility: Visibility,
    pub(super) exclusion: DdlExclusion,
}
redacted!(PreflightRequest<'_>);
impl<'p> PreflightRequest<'p> {
    /// Pure: checks finite capability and explicit scope, without connecting.
    pub fn new(
        plan: &'p MigrationPlan,
        phase: MigrationPhase,
        scope: Scope,
        visibility: Visibility,
        exclusion: DdlExclusion,
    ) -> std::result::Result<Self, PreflightError> {
        let p = plan
            .phases()
            .into_iter()
            .find(|p| p.phase() == phase)
            .expect("fixed phase");
        match p.capability() {
            PhaseCapability::Unsupported => return Err(PreflightError::Refused),
            PhaseCapability::NoWork => return Err(PreflightError::NoWork),
            PhaseCapability::NullableAppend => {}
        }
        let sensitive = plan.sensitive_source();
        let (before, after) = sensitive.schemas();
        if plan.public_view().status() != MigrationStatus::SqlPlanned
            || before.authority() != SchemaAuthority::ApplicationDeclared
            || after.authority() != SchemaAuthority::ApplicationDeclared
        {
            return Err(PreflightError::Refused);
        }
        let tables = before.sensitive_tables();
        if tables.len() != scope.tables.len()
            || tables.iter().any(|t| {
                let (ns, name) = t.sensitive_name().sensitive_parts();
                ns.sensitive_text() != scope.namespace
                    || !scope.tables.iter().any(|s| s == name.sensitive_text())
            })
        {
            return Err(PreflightError::Mismatch);
        }
        if exclusion == DdlExclusion::Uncontrolled {
            return Err(PreflightError::Incomplete);
        }
        Ok(Self {
            plan,
            phase,
            scope,
            visibility,
            exclusion,
        })
    }
    pub(super) fn same(&self, other: &Self) -> bool {
        self.plan.same_record(other.plan)
            && self.phase == other.phase
            && self.scope == other.scope
            && self.visibility == other.visibility
            && self.exclusion == other.exclusion
    }
    pub(super) fn retained(&self) -> Self {
        Self {
            plan: self.plan,
            phase: self.phase,
            scope: self.scope.clone(),
            visibility: self.visibility.clone(),
            exclusion: self.exclusion,
        }
    }
    // A private finite expected-column projection only. It is never converted
    // back into MigrationSchema or exported as live/full-domain authority.
    pub(super) fn expected(&self) -> std::result::Result<Schema, PreflightError> {
        let sensitive = self.plan.sensitive_source();
        let (before, _) = sensitive.schemas();
        let (database, address, catalog, dialect) = before.sensitive_target().sensitive_parts();
        let mut tables = Vec::new();
        for t in before.sensitive_tables() {
            if t.constraint_counts() != [0; 3] {
                return Err(PreflightError::Unsupported);
            }
            let (ns, name) = t.sensitive_name().sensitive_parts();
            let mut columns = Vec::new();
            for c in t.sensitive_columns() {
                let (name, native, nullability, default) = c.sensitive_parts();
                let native = match native {
                    Observed::Known(SqlType::I8) => Native::I8,
                    Observed::Known(SqlType::I16) => Native::I16,
                    Observed::Known(SqlType::I32) => Native::I32,
                    Observed::Known(SqlType::I64) => Native::I64,
                    _ => return Err(PreflightError::Unsupported),
                };
                let nullability = match nullability {
                    Observed::Known(true) => Nullability::Nullable,
                    Observed::Known(false) => Nullability::Nonnull,
                    _ => return Err(PreflightError::Incomplete),
                };
                let default = match default {
                    MigrationDefault::Absent => DefaultValue::None {},
                    MigrationDefault::Literal(Value::I8(v)) => DefaultValue::Literal {
                        value: Literal::I8(*v),
                    },
                    MigrationDefault::Literal(Value::I16(v)) => DefaultValue::Literal {
                        value: Literal::I16(*v),
                    },
                    MigrationDefault::Literal(Value::I32(v)) => DefaultValue::Literal {
                        value: Literal::I32(*v),
                    },
                    MigrationDefault::Literal(Value::I64(v)) => DefaultValue::Literal {
                        value: Literal::I64(*v),
                    },
                    _ => return Err(PreflightError::Incomplete),
                };
                columns.push(Column {
                    name: name.sensitive_text().into(),
                    native,
                    nullability,
                    default: Some(default),
                });
            }
            tables.push(Table {
                path: vec![ns.sensitive_text().into(), name.sensitive_text().into()],
                semantics: Semantics::Ordinary,
                columns_coverage: Coverage::Complete,
                keys_coverage: Coverage::Complete,
                columns,
                keys: vec![],
            });
        }
        let mut schema = Schema {
            kind: SchemaKind::QuentSchema,
            version: 1,
            target: Target {
                database: database.into(),
                address: address.into(),
                catalog: catalog.sensitive_text().into(),
                dialect: match dialect {
                    quent_core::Dialect::Postgres => Dialect::Postgres,
                    quent_core::Dialect::Mysql => Dialect::Mysql,
                },
            },
            coverage: Coverage::Complete,
            tables,
        };
        normalize(&mut schema);
        Ok(schema)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightError {
    Refused,
    NoWork,
    Incomplete,
    Unsupported,
    Mismatch,
    Drift,
    Read(VerifyError),
}
impl From<VerifyError> for PreflightError {
    fn from(e: VerifyError) -> Self {
        Self::Read(e)
    }
}
impl fmt::Display for PreflightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_PREFLIGHT_{self:?}")
    }
}
impl std::error::Error for PreflightError {}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightState {
    ConditionsObserved,
    Mismatch,
    Stale,
    Unverified,
}
/// Output only. Partial overall coverage explicitly retains unobserved full
/// effects/source syntax; ConditionsObserved is NOT Ready or an apply permit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreflightView {
    state: PreflightState,
    steps: usize,
    tables: usize,
}
impl PreflightView {
    pub fn state(self) -> PreflightState {
        self.state
    }
    pub fn to_json(self) -> String {
        format!(
            "{{\"kind\":\"quent_migration_preflight_view\",\"version\":1,\"state\":\"{:?}\",\"coverage\":\"Partial\",\"phase\":\"Expand\",\"steps\":{},\"tables\":{},\"conditions\":3,\"full_effects\":\"Unverified\",\"authority\":\"ApplicationDeclared\"}}",
            self.state, self.steps, self.tables
        )
    }
}
impl fmt::Display for PreflightView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
/// Private, non-Clone, non-serialized successful finite observation. Recheck is
/// repeatable within TTL; there is no executable handle or consumption authority.
/// ```compile_fail
/// let e = quent::manifest::live::preflight::PreflightObservation {};
/// ```
pub struct PreflightObservation<'p> {
    pub(super) request: PreflightRequest<'p>,
    pub(super) context: DatabaseContext,
    pub(super) identity: Arc<()>,
    pub(super) observed: Schema,
    pub(super) extension: Vec<Vec<Option<String>>>,
    pub(super) started: Instant,
    pub(super) completed: Instant,
    pub(super) deadline: Instant,
    pub(super) wall_started_ms: u64,
    pub(super) wall_completed_ms: u64,
}
redacted!(PreflightObservation<'_>);
impl PreflightObservation<'_> {
    pub(super) fn compare(
        &self,
        available: bool,
        identity: &Arc<()>,
        current: &DatabaseContext,
        request: &PreflightRequest<'_>,
    ) -> PreflightView {
        let mut view = PreflightView {
            state: PreflightState::Unverified,
            steps: request.plan.public_view().steps(),
            tables: request.scope.tables.len(),
        };
        if !available {
            return view;
        }
        if !Arc::ptr_eq(identity, &self.identity)
            || !current.same_snapshot(&self.context)
            || !request.same(&self.request)
        {
            view.state = PreflightState::Mismatch;
            return view;
        }
        let now = Instant::now();
        if now < self.completed
            || self.completed < self.started
            || now >= self.deadline
            || wall().map_or(true, |w| {
                w < self.wall_completed_ms || self.wall_completed_ms < self.wall_started_ms
            })
        {
            view.state = PreflightState::Stale;
            return view;
        }
        // Repeat exact measured result/expected checks, never public claims.
        if matches_result(request, &self.observed, &self.extension).is_err() {
            return view;
        }
        let final_now = Instant::now();
        view.state = if final_now < now
            || final_now >= self.deadline
            || wall().map_or(true, |w| w < self.wall_completed_ms)
        {
            PreflightState::Stale
        } else {
            PreflightState::ConditionsObserved
        };
        view
    }
}
pub(super) fn matches_result(
    request: &PreflightRequest<'_>,
    observed: &Schema,
    extension: &[Vec<Option<String>>],
) -> std::result::Result<(), PreflightError> {
    if extension.len() != request.scope.tables.len() {
        return Err(PreflightError::Incomplete);
    }
    for (row, table) in extension.iter().zip(&request.scope.tables) {
        if row.len() != 6
            || field(row, 0)? != request.scope.namespace
            || field(row, 1)? != table
            || field(row, 2)? != request.visibility.role
        {
            return Err(PreflightError::Mismatch);
        }
        if field(row, 3)? != "0" || field(row, 4)? != "0" || field(row, 5)? != "false" {
            return Err(PreflightError::Incomplete);
        }
    }
    // Unknown default is never equality with Absent; normalized NULL cannot prove syntax.
    if schema_coverage(observed) != SourceCoverage::Complete {
        return Err(PreflightError::Incomplete);
    }
    let expected = request.expected()?;
    if *observed != expected {
        return Err(PreflightError::Drift);
    }
    Ok(())
}

// Expanded within the existing closed adapters: same owned connection, quiet
// polling/drop, fixed read-only transaction and one shared Meter/timeout.
macro_rules! preflight_reader {
    ($reader:ident) => {
        impl $reader {
            pub async fn observe_migration<'p>(
                &mut self,
                current: &DatabaseContext,
                request: &super::preflight::PreflightRequest<'p>,
                budget: Budget,
                ttl: Duration,
            ) -> std::result::Result<
                super::preflight::PreflightObservation<'p>,
                super::preflight::PreflightError,
            > {
                use super::preflight::*;
                let b = budget.check()?;
                let expected = request.expected()?;
                if !self.available
                    || !self.context.same_snapshot(current)
                    || expected.target != self.target
                {
                    return Err(PreflightError::Mismatch);
                }
                if request.scope.tables.len() > b.tables || request.scope.tables.len() > b.branches
                {
                    return Err(PreflightError::Read(VerifyError::Budget));
                }
                let started = Instant::now();
                let deadline = expiry(started, ttl)?;
                let wall_started_ms = wall()?;
                let io_deadline = started.checked_add(b.timeout).ok_or(VerifyError::Clock)?;
                self.available = false;
                let connection = self.connection.as_mut().ok_or(VerifyError::Connect)?;
                let mut meter = Meter {
                    b,
                    queries: 0,
                    rows: 0,
                    bytes: 0,
                    #[cfg(test)]
                    trace: self.trace.clone(),
                    #[cfg(test)]
                    disconnect_after_header: self.disconnect_after_header,
                };
                let mut extension = Vec::new();
                let observed = quiet(tokio::time::timeout(
                    b.timeout,
                    read(
                        connection,
                        &self.target,
                        &request.scope,
                        &mut meter,
                        Some(&mut extension),
                    ),
                ))
                .await
                .map_err(|_| VerifyError::Expired)??;
                // ACK cleanup alone never upgrades partial/drift into evidence.
                matches_result(request, &observed, &extension)?;
                let completed = Instant::now();
                let wall_completed_ms = wall()?;
                if completed < started
                    || completed >= deadline
                    || completed >= io_deadline
                    || wall_completed_ms < wall_started_ms
                {
                    return Err(PreflightError::Read(VerifyError::Expired));
                }
                let observation = PreflightObservation {
                    request: request.retained(),
                    context: current.clone(),
                    identity: self.identity.clone(),
                    observed,
                    extension,
                    started,
                    completed,
                    deadline,
                    wall_started_ms,
                    wall_completed_ms,
                };
                // Independently sample time/current input again at the return boundary.
                let view = observation.compare(true, &self.identity, current, request);
                if view.state() != PreflightState::ConditionsObserved
                    || Instant::now() >= io_deadline
                {
                    return Err(PreflightError::Read(VerifyError::Expired));
                }
                self.available = true;
                Ok(observation)
            }
            /// Independent current input and process clock recheck; no new I/O.
            /// This is not continuous live drift monitoring or an apply gate.
            pub fn compare_migration(
                &self,
                current: &DatabaseContext,
                request: &super::preflight::PreflightRequest<'_>,
                evidence: &super::preflight::PreflightObservation<'_>,
            ) -> super::preflight::PreflightView {
                evidence.compare(self.available, &self.identity, current, request)
            }
        }
    };
}
pub(super) use preflight_reader;
#[cfg(test)]
mod tests;
