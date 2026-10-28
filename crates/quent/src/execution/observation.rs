//! Explicit application-owned bounded observation. No ambient context or exporter.
use super::*;
use std::sync::Mutex;

const MAX_ENTRIES: usize = 128;
const MAX_SUBMISSIONS: u16 = 4096;
const MAX_MICROS: u64 = 300_000_000;

/// Limits of the local collector, not server resources or total wire traffic.
#[derive(Clone, Copy)]
pub struct ObservationBudget {
    submissions: u16,
    entries: u16,
    duration: Duration,
    repeat: u16,
}
impl ObservationBudget {
    /// Non-Tx SQL attempts 0..4096; entries 1..128; per-operation measurement
    /// ceiling (0,300s]; repeat suspicion threshold 2..128.
    pub fn new(submissions: u16, entries: u16, duration: Duration, repeat: u16) -> Result<Self> {
        if submissions > MAX_SUBMISSIONS
            || !(1..=MAX_ENTRIES as u16).contains(&entries)
            || duration.is_zero()
            || duration > Duration::from_secs(300)
            || !(2..=MAX_ENTRIES as u16).contains(&repeat)
        {
            return Err(ExecutionError::ObservationLimit);
        }
        Ok(Self {
            submissions,
            entries,
            duration,
            repeat,
        })
    }
}
impl fmt::Debug for ObservationBudget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ObservationBudget([REDACTED])")
    }
}
/// An explicit application request bound to one entire immutable context snapshot.
/// Sequential session scopes only; no Clone, serde, reset, refund or shared handle.
/// Construction authenticates no HTTP request or physical server.
pub struct RequestCollector {
    pub(super) shared: Shared,
}
impl fmt::Debug for RequestCollector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RequestCollector([REDACTED])")
    }
}
impl RequestCollector {
    pub fn new(current: &DatabaseContext, budget: ObservationBudget) -> Self {
        Self {
            shared: Arc::new(Mutex::new(State {
                context: current.clone(),
                budget,
                view: ObservationView {
                    submissions: 0,
                    incomplete: false,
                    closed: false,
                    budget_refused: false,
                    capacity_exceeded: false,
                    entries: Vec::new(),
                },
                shapes: Vec::with_capacity(usize::from(budget.entries)),
            })),
        }
    }
    pub fn public_view(&self) -> ObservationView {
        let mut view = lock(&self.shared).view.clone();
        if view
            .entries
            .iter()
            .any(|e| e.status == ObservationStatus::Pending)
        {
            view.incomplete = true;
        }
        view
    }
    /// End the local request. Private evidence cannot restore execution authority.
    pub fn finish(self) -> QueryObservation {
        let mut state = lock(&self.shared);
        state.view.closed = true;
        let mut view = state.view.clone();
        if view
            .entries
            .iter()
            .any(|e| e.status == ObservationStatus::Pending)
        {
            view.incomplete = true;
        }
        QueryObservation {
            _context: state.context.clone(),
            _shapes: std::mem::take(&mut state.shapes),
            view,
        }
    }
}
/// Private nonrestorable retained evidence. Shape identifiers never leave it.
pub struct QueryObservation {
    _context: DatabaseContext,
    _shapes: Vec<Shape>,
    view: ObservationView,
}
impl QueryObservation {
    pub fn public_view(&self) -> ObservationView {
        self.view.clone()
    }
}
impl fmt::Debug for QueryObservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("QueryObservation([REDACTED])")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationStatus {
    Pending,
    Succeeded,
    Failed,
    Refused,
    Interrupted,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consumption {
    NotObserved,
    Partial,
    Complete,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedTransaction {
    None,
    BorrowedUnconfirmed,
    NotStarted,
    Unknown,
    Committed,
    RolledBack,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationPrecision {
    Exact,
    CeilingExceeded,
}
/// Fixed enums and bounded numbers only. Not a SQL/result/authority container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryObservationView {
    pub status: ObservationStatus,
    pub consumption: Consumption,
    pub transaction: ObservedTransaction,
    pub duration_micros: u64,
    pub duration_precision: DurationPrecision,
    pub submitted: bool,
    pub decoded_rows: u16,
    pub decoded_bytes: u32,
    pub repeat_count: u16,
    pub n_plus_one_suspected: bool,
    pub begin_attempts: u8,
    pub begin_acks: u8,
    pub commit_attempts: u8,
    pub commit_acks: u8,
    pub rollback_attempts: u8,
    pub rollback_acks: u8,
}
/// Output only; no Deserialize/parser or conversion to request/permit/operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationView {
    submissions: u16,
    incomplete: bool,
    closed: bool,
    budget_refused: bool,
    capacity_exceeded: bool,
    entries: Vec<QueryObservationView>,
}
impl ObservationView {
    pub fn submissions(&self) -> u16 {
        self.submissions
    }
    pub fn is_incomplete(&self) -> bool {
        self.incomplete
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    pub fn entries(&self) -> &[QueryObservationView] {
        &self.entries
    }
    pub fn to_json(&self) -> String {
        use std::fmt::Write;
        let mut s = format!(
            "{{\"kind\":\"quent_observation_view\",\"version\":1,\"submissions\":{},\"incomplete\":{},\"closed\":{},\"budget_refused\":{},\"capacity_exceeded\":{},\"entries\":[",
            self.submissions,
            self.incomplete,
            self.closed,
            self.budget_refused,
            self.capacity_exceeded
        );
        for (i, e) in self.entries.iter().enumerate() {
            if i != 0 {
                s.push(',');
            }
            // Debug is used only for this closed local enum register, never errors/data.
            write!(s,"{{\"status\":\"{:?}\",\"consumption\":\"{:?}\",\"transaction\":\"{:?}\",\"duration_micros\":{},\"duration_precision\":\"{:?}\",\"submitted\":{},\"decoded_rows\":{},\"decoded_bytes\":{},\"repeat_count\":{},\"n_plus_one_suspected\":{},\"begin_attempts\":{},\"begin_acks\":{},\"commit_attempts\":{},\"commit_acks\":{},\"rollback_attempts\":{},\"rollback_acks\":{}}}", e.status,e.consumption,e.transaction,e.duration_micros,e.duration_precision,e.submitted,e.decoded_rows,e.decoded_bytes,e.repeat_count,e.n_plus_one_suspected,e.begin_attempts,e.begin_acks,e.commit_attempts,e.commit_acks,e.rollback_attempts,e.rollback_acks).expect("String write");
        }
        s.push_str("]}");
        s
    }
}
impl fmt::Display for ObservationView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}

pub(super) type Shared = Arc<Mutex<State>>;
pub(super) struct State {
    context: DatabaseContext,
    budget: ObservationBudget,
    view: ObservationView,
    shapes: Vec<Shape>,
}
fn lock(s: &Shared) -> std::sync::MutexGuard<'_, State> {
    s.lock().unwrap_or_else(|p| {
        let mut state = p.into_inner();
        state.view.incomplete = true;
        state.view.closed = true;
        state
    })
}
#[derive(Clone)]
pub(super) struct Link {
    pub shared: Shared,
    pub active: Option<usize>,
}
pub(super) fn attach(shared: &Shared, context: &DatabaseContext) -> Result<Link> {
    let mut s = lock(shared);
    if s.view
        .entries
        .iter()
        .any(|e| e.status == ObservationStatus::Pending)
    {
        s.view.incomplete = true;
        s.view.closed = true;
    }
    if !s.context.same_snapshot(context) {
        return Err(ExecutionError::ContextMismatch);
    }
    if s.view.closed {
        return Err(ExecutionError::ObservationClosed);
    }
    Ok(Link {
        shared: shared.clone(),
        active: None,
    })
}
pub(super) fn unsupported(link: &Option<Link>) -> Result<()> {
    if let Some(l) = link {
        let mut s = lock(&l.shared);
        s.view.incomplete = true;
        s.view.closed = true;
        return Err(ExecutionError::Unsupported);
    }
    Ok(())
}
// Exact structural comparison over the immutable final IR; no SQL text or hash.
// Literal values are omitted, but their native types/NULL status and IN arity remain.
#[derive(PartialEq, Eq)]
struct Kind(SqlType, bool);
fn kind(v: &Value) -> Kind {
    Kind(v.sql_type(), matches!(v, Value::Null(_)))
}
#[derive(PartialEq, Eq)]
enum Pred {
    And(Vec<Pred>),
    Or(Vec<Pred>),
    Not(Box<Pred>),
    Compare(Column, Compare, Kind),
    In(Column, Vec<Kind>),
    Null(Column),
}
fn pred(p: &Predicate) -> Pred {
    match p {
        Predicate::And(v) => Pred::And(v.iter().map(pred).collect()),
        Predicate::Or(v) => Pred::Or(v.iter().map(pred).collect()),
        Predicate::Not(v) => Pred::Not(Box::new(pred(v))),
        Predicate::Compare(c, o, v) => Pred::Compare(c.clone(), *o, kind(v)),
        Predicate::In(c, v) => Pred::In(c.clone(), v.iter().map(kind).collect()),
        Predicate::IsNull(c) => Pred::Null(c.clone()),
    }
}
#[derive(PartialEq, Eq)]
struct Shape {
    table: Table,
    family: Family,
    projection: FinalProjection,
    predicate: Option<Pred>,
    write: Vec<(Column, Option<Kind>)>,
    order: Vec<Order>,
    limit: bool,
    version: Option<(Column, Kind)>,
    args: Vec<Kind>,
}
impl Shape {
    fn new(source: &SensitiveSource<'_>) -> Self {
        let m = source.metadata();
        Self {
            table: source.sensitive_table().clone(),
            family: m.family(),
            projection: m.projection().clone(),
            predicate: m.predicate().map(pred),
            write: m
                .ordered_write_data()
                .iter()
                .map(|(c, v)| {
                    (
                        c.clone(),
                        match v {
                            Input::Value(v) => Some(kind(v)),
                            Input::Missing => None,
                        },
                    )
                })
                .collect(),
            order: m.order().to_vec(),
            limit: m.limit().is_some(),
            version: m.sensitive_version().map(|(c, v)| (c.clone(), kind(v))),
            args: source.args().iter().map(kind).collect(),
        }
    }
}
pub(super) struct PreparedShape {
    shape: Shape,
    repeats: u16,
}
pub(super) struct Attempt {
    link: Link,
    index: usize,
    start: Instant,
    owner: bool,
    done: bool,
}
impl Attempt {
    pub fn start(link: &Option<Link>, tx: ObservedTransaction) -> Result<Option<Self>> {
        let Some(l) = link else { return Ok(None) };
        if let Some(index) = l.active {
            return Ok(Some(Self {
                link: l.clone(),
                index,
                start: Instant::now(),
                owner: false,
                done: false,
            }));
        }
        let mut s = lock(&l.shared);
        if s.view.closed {
            return Err(ExecutionError::ObservationClosed);
        }
        // A deliberately forgotten scope can end Rust's request borrow while its
        // private attachment remains in the original session. Enforce sequential
        // attempts here too, before begin/send, rather than relying only on attach.
        if s.view
            .entries
            .iter()
            .any(|e| e.status == ObservationStatus::Pending)
        {
            s.view.incomplete = true;
            s.view.closed = true;
            return Err(ExecutionError::ObservationClosed);
        }
        if s.view.entries.len() == usize::from(s.budget.entries) {
            s.view.capacity_exceeded = true;
            s.view.incomplete = true;
            s.view.closed = true;
            return Err(ExecutionError::ObservationLimit);
        }
        let index = s.view.entries.len();
        s.view.entries.push(QueryObservationView {
            status: ObservationStatus::Pending,
            consumption: Consumption::NotObserved,
            transaction: tx,
            duration_micros: 0,
            duration_precision: DurationPrecision::Exact,
            submitted: false,
            decoded_rows: 0,
            decoded_bytes: 0,
            repeat_count: 0,
            n_plus_one_suspected: false,
            begin_attempts: 0,
            begin_acks: 0,
            commit_attempts: 0,
            commit_acks: 0,
            rollback_attempts: 0,
            rollback_acks: 0,
        });
        Ok(Some(Self {
            link: l.clone(),
            index,
            start: Instant::now(),
            owner: true,
            done: false,
        }))
    }
    pub fn inherited(&self) -> Link {
        Link {
            shared: self.link.shared.clone(),
            active: Some(self.index),
        }
    }
    pub fn update(&self, f: impl FnOnce(&mut QueryObservationView)) {
        f(&mut lock(&self.link.shared).view.entries[self.index]);
    }
    pub fn prepare_shape(&self, source: &SensitiveSource<'_>) -> PreparedShape {
        let shape = Shape::new(source);
        let s = lock(&self.link.shared);
        let repeats = 1 + s.shapes.iter().filter(|p| **p == shape).count() as u16;
        PreparedShape { shape, repeats }
    }
    pub fn charge(&self, prepared: PreparedShape) -> Result<()> {
        let mut s = lock(&self.link.shared);
        if s.view.closed {
            return Err(ExecutionError::ObservationClosed);
        }
        if s.view.submissions == s.budget.submissions {
            s.view.budget_refused = true;
            s.view.closed = true;
            return Err(ExecutionError::DispatchBudget);
        }
        let threshold = s.budget.repeat;
        let PreparedShape { shape, repeats } = prepared;
        s.shapes.push(shape);
        s.view.submissions += 1;
        let e = &mut s.view.entries[self.index];
        e.submitted = true;
        e.repeat_count = repeats;
        e.n_plus_one_suspected = repeats >= threshold;
        Ok(())
    }
    pub fn result(&self, result: &Result<ExecutionResult>) {
        self.update(|e| {
            if result.is_ok() {
                e.consumption = Consumption::Complete;
            }
        });
    }
    pub fn finish(&mut self, success: bool) {
        if !self.owner {
            self.done = true;
            return;
        }
        let elapsed = self.start.elapsed();
        let mut s = lock(&self.link.shared);
        let ceiling = s.budget.duration;
        let e = &mut s.view.entries[self.index];
        e.status = if success {
            ObservationStatus::Succeeded
        } else if e.submitted || e.begin_attempts > 0 {
            ObservationStatus::Failed
        } else {
            ObservationStatus::Refused
        };
        e.duration_precision = if elapsed > ceiling {
            DurationPrecision::CeilingExceeded
        } else {
            DurationPrecision::Exact
        };
        e.duration_micros = elapsed.min(ceiling).as_micros().min(u128::from(MAX_MICROS)) as u64;
        if !success || elapsed > ceiling {
            s.view.closed = true;
            s.view.incomplete = true;
        }
        self.done = true;
    }
}
impl Drop for Attempt {
    fn drop(&mut self) {
        if self.owner && !self.done {
            let mut s = lock(&self.link.shared);
            s.view.incomplete = true;
            s.view.closed = true;
            let elapsed = self.start.elapsed();
            let ceiling = s.budget.duration;
            let e = &mut s.view.entries[self.index];
            e.status = ObservationStatus::Interrupted;
            e.duration_micros = elapsed.min(ceiling).as_micros() as u64;
            e.duration_precision = if elapsed > ceiling {
                DurationPrecision::CeilingExceeded
            } else {
                DurationPrecision::Exact
            };
            // Destruction supplies no completion ACK and cannot prove cancellation at server.
        }
    }
}

// Scopes expose only finite methods, never DerefMut/AsMut to the session: a
// caller must not swap in an unobserved session through an attached wrapper.
macro_rules! scope_methods {
    () => {
        pub fn is_available(&self) -> bool {
            self.session.is_available()
        }
        pub fn validate(
            &self,
            c: &DatabaseContext,
            o: &Operation,
            ttl: Duration,
        ) -> Result<ValidatedOperation> {
            self.session.validate(c, o, ttl)
        }
        pub async fn execute(
            &mut self,
            c: &DatabaseContext,
            o: Operation,
        ) -> Result<ExecutionResult> {
            self.session.execute(c, o).await
        }
        pub async fn execute_validated(
            &mut self,
            c: &DatabaseContext,
            o: &Operation,
            h: ValidatedOperation,
        ) -> Result<ExecutionResult> {
            self.session.execute_validated(c, o, h).await
        }

        pub fn validate_atomic(
            &self,
            c: &DatabaseContext,
            o: &Operation,
            e: ExpectedRows,
            t: Duration,
        ) -> Result<ValidatedAtomicOperation> {
            self.session.validate_atomic(c, o, e, t)
        }
    };
}
macro_rules! observation_scope {
    ($session:ident,$scope:ident) => {
        /// Exclusive local request scope. No raw session conversion or replacement.
        pub struct $scope<'a> {
            session: &'a mut $session,
            _request: &'a mut RequestCollector,
        }
        impl fmt::Debug for $scope<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($scope), "([REDACTED])"))
            }
        }
        impl $scope<'_> {
            scope_methods!();
            #[cfg(feature = "operation-spec")]
            pub async fn execute_compiled_read(
                &mut self,
                c: &DatabaseContext,
                m: &crate::operation_spec::TypedModelContract,
                r: crate::operation_spec::CompiledRead,
            ) -> Result<ExecutionResult> {
                self.session.execute_compiled_read(c, m, r).await
            }
            pub async fn execute_atomic(
                &mut self,
                c: &DatabaseContext,
                o: &Operation,
                e: ExpectedRows,
                h: ValidatedAtomicOperation,
            ) -> std::result::Result<ExecutionResult, AtomicFailure> {
                self.session.execute_atomic(c, o, e, h).await
            }
        }
        impl Drop for $scope<'_> {
            fn drop(&mut self) {
                self.session.observation = None;
            }
        }
        impl $session {
            pub fn observe<'a>(
                &'a mut self,
                request: &'a mut RequestCollector,
            ) -> Result<$scope<'a>> {
                if self.observation.is_some() {
                    return Err(ExecutionError::Unsupported);
                }
                self.observation = Some(observation::attach(&request.shared, &self.context)?);
                Ok($scope {
                    session: self,
                    _request: request,
                })
            }
        }
    };
}
macro_rules! borrowed_scope {
    ($session:ident,$scope:ident) => {
        /// Explicit request observation on caller-owned Tx; completion is unobserved.
        pub struct $scope<'a, 'tx> {
            session: &'a mut $session<'tx>,
            _request: &'a mut RequestCollector,
        }
        impl fmt::Debug for $scope<'_, '_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($scope), "([REDACTED])"))
            }
        }
        impl $scope<'_, '_> {
            scope_methods!();
        }
        impl Drop for $scope<'_, '_> {
            fn drop(&mut self) {
                self.session.observation = None;
            }
        }
        impl<'tx> $session<'tx> {
            pub fn observe<'a>(
                &'a mut self,
                request: &'a mut RequestCollector,
            ) -> Result<$scope<'a, 'tx>> {
                if self.observation.is_some() {
                    return Err(ExecutionError::Unsupported);
                }
                self.observation = Some(observation::attach(&request.shared, &self.context)?);
                Ok($scope {
                    session: self,
                    _request: request,
                })
            }
        }
    };
}
#[cfg(feature = "postgres")]
observation_scope!(PostgresSession, ObservedPostgresSession);
#[cfg(feature = "mysql")]
observation_scope!(MysqlSession, ObservedMysqlSession);
#[cfg(feature = "postgres")]
borrowed_scope!(
    BorrowedPostgresTransaction,
    ObservedBorrowedPostgresTransaction
);
#[cfg(feature = "mysql")]
borrowed_scope!(BorrowedMysqlTransaction, ObservedBorrowedMysqlTransaction);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_repeat_shape_uses_exact_structure_without_literal_values() {
        let table = |namespace: &str, alias: Option<&str>| {
            Table::new(
                vec![
                    Ident::new(namespace).unwrap(),
                    Ident::new("CANARY_table").unwrap(),
                ],
                alias.map(|a| Ident::new(a).unwrap()),
            )
            .unwrap()
        };
        let make = |t: Table, value: i64, comparison: Compare| {
            let c = t.column(Ident::new("CANARY_column").unwrap(), SqlType::I64, false);
            Draft::select(t, vec![Projection::new(c.clone(), None)])
                .filter(Predicate::Compare(c, comparison, Value::I64(value)))
                .freeze()
                .finalize(Dialect::Postgres)
                .unwrap()
        };
        let first = make(table("CANARY_namespace", None), 1, Compare::Eq);
        let changed_value = make(table("CANARY_namespace", None), 99, Compare::Eq);
        assert!(
            Shape::new(&first.sensitive_source()) == Shape::new(&changed_value.sensitive_source())
        );
        for other in [
            make(table("other_namespace", None), 1, Compare::Eq),
            make(table("CANARY_namespace", Some("alias")), 1, Compare::Eq),
            make(table("CANARY_namespace", None), 1, Compare::Gt),
        ] {
            assert!(Shape::new(&first.sensitive_source()) != Shape::new(&other.sensitive_source()));
        }
        let t = table("CANARY_namespace", None);
        let c = t.column(Ident::new("nullable").unwrap(), SqlType::I64, true);
        let make = |values: Vec<Value>| {
            Draft::select(t.clone(), vec![Projection::new(c.clone(), None)])
                .filter(Predicate::In(c.clone(), values))
                .freeze()
                .finalize(Dialect::Postgres)
                .unwrap()
        };
        let a = make(vec![Value::I64(1)]);
        let b = make(vec![Value::I64(2)]);
        let null = make(vec![Value::Null(SqlType::I64)]);
        let two = make(vec![Value::I64(1), Value::I64(2)]);
        assert!(Shape::new(&a.sensitive_source()) == Shape::new(&b.sensitive_source()));
        for p in [null, two] {
            assert!(Shape::new(&a.sensitive_source()) != Shape::new(&p.sensitive_source()));
        }
    }
}
