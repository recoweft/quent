use super::*;

/// Assertions about an ordinary base table. Unsupported includes views, triggers,
/// RLS, generated semantics and any unverified/unknown table context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableContext {
    OrdinaryBaseTableWithoutEffects,
    Unsupported,
}
/// Both primary and unique constraints require all nonnullable signed components.
#[derive(Clone, PartialEq, Eq)]
pub enum Key {
    Primary(Vec<Column>),
    Unique(Vec<Column>),
}
impl Key {
    fn columns(&self) -> &[Column] {
        match self {
            Self::Primary(c) | Self::Unique(c) => c,
        }
    }
}
/// Owned application assertion, never a live catalog observation or authority
/// derived from a name, public digest, fixture, freshness or allow claim.
#[derive(Clone, PartialEq, Eq)]
pub struct ScopeFacts {
    pub(super) table: Table,
    pub(super) dialect: Dialect,
    pub(super) columns: Vec<Column>,
    keys: Vec<Key>,
    context: TableContext,
}
impl ScopeFacts {
    pub fn application_asserted(
        table: Table,
        dialect: Dialect,
        columns: Vec<Column>,
        keys: Vec<Key>,
        context: TableContext,
    ) -> Result<Self> {
        if columns.len() > 256 || keys.len() > 16 {
            return Err(PlanError::Budget);
        }
        for (i, c) in columns.iter().enumerate() {
            if c.table != table || columns[..i].iter().any(|p| p.name == c.name) {
                return Err(PlanError::Shape);
            }
        }
        for key in &keys {
            let cs = key.columns();
            if cs.is_empty() {
                return Err(PlanError::Shape);
            }
            if cs.len() > 16 {
                return Err(PlanError::Budget);
            }
            for (i, c) in cs.iter().enumerate() {
                if !columns.contains(c) || cs[..i].contains(c) {
                    return Err(PlanError::Shape);
                }
            }
        }
        Ok(Self {
            table,
            dialect,
            columns,
            keys,
            context,
        })
    }
}
redacted!(Key, ScopeFacts);

/// Maximum matching target rows under the stated assumptions; never affected
/// rows, an estimate, authorization, or a bound on trigger/transaction effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Zero,
    Single,
    FiniteBound(u16),
    Broad,
    Unknown,
    NotApplicable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completeness {
    Complete,
    Partial,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assumptions {
    None,
    ApplicationAsserted,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evidence {
    MissingFacts,
    InconsistentFacts,
    UnsupportedContext,
    UnsupportedKey,
    AnalysisBudget,
    EmptyTruthSet,
    CompleteKeyTuples,
    UnboundedPredicate,
    UnsupportedPredicate,
    NoPredicate,
    InsertShape,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Risk {
    Low,
    Medium,
    High,
    Destructive,
    Blocked,
}
/// Fixed diagnostic register. No caller reason, source errors or suppression API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingCode {
    MissingWhere,
    UnknownProof,
    BulkWrite,
    WeakPredicate,
    ReadWithoutLimit,
    ReadProjection,
    InsertUnverified,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finding {
    code: FindingCode,
}
impl Finding {
    pub fn code(&self) -> FindingCode {
        self.code
    }
    pub fn suppressible(&self) -> bool {
        false
    }
}
/// Immutable inspection tied by borrow to the very finalized record inspected.
/// No execute, permit, mutable public claim or reconstruction input.
/// ```compile_fail
/// fn forge(i: &mut quent_core::Inspection<'_>) { i.risk = quent_core::Risk::Low; }
/// ```
/// ```compile_fail
/// fn execute(i: quent_core::Inspection<'_>) { i.execute(); }
/// ```
pub struct Inspection<'a> {
    record: &'a Record,
    scope: Scope,
    completeness: Completeness,
    assumptions: Assumptions,
    evidence: Evidence,
    risk: Risk,
    findings: Vec<Finding>,
}
impl Inspection<'_> {
    pub fn scope(&self) -> Scope {
        self.scope
    }
    pub fn completeness(&self) -> Completeness {
        self.completeness
    }
    pub fn assumptions(&self) -> Assumptions {
        self.assumptions
    }
    pub fn evidence(&self) -> Evidence {
        self.evidence
    }
    pub fn risk(&self) -> Risk {
        self.risk
    }
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
    pub fn sensitive_source(&self) -> SensitiveSource<'_> {
        SensitiveSource {
            record: self.record,
        }
    }
}
redacted!(Inspection<'_>);

// A capped expansion of TRUE candidates, not a SQL re-parser, renderer or SAT
// solver. UNKNOWN and FALSE both fail WHERE; NOT never complements this abstraction.
#[derive(Clone, Default)]
struct Branch {
    equalities: BTreeMap<Ident, i64>,
    partial: bool,
}
fn signed(v: &Value) -> Option<i64> {
    match v {
        Value::I8(n) => Some((*n).into()),
        Value::I16(n) => Some((*n).into()),
        Value::I32(n) => Some((*n).into()),
        Value::I64(n) => Some(*n),
        _ => None,
    }
}
fn signed_column(c: &Column) -> bool {
    !c.nullable
        && matches!(
            c.sql_type,
            SqlType::I8 | SqlType::I16 | SqlType::I32 | SqlType::I64
        )
}
fn branches(p: &Predicate, work: &mut usize) -> Option<Vec<Branch>> {
    let unknown = || {
        vec![Branch {
            partial: true,
            ..Branch::default()
        }]
    };
    let result = match p {
        Predicate::Or(ps) => {
            let mut out = vec![];
            for p in ps {
                out.extend(branches(p, work)?);
                if out.len() > 256 {
                    return None;
                }
            }
            out
        }
        Predicate::And(ps) => {
            let mut out = vec![Branch::default()];
            for p in ps {
                let next = branches(p, work)?;
                let mut joined = vec![];
                for a in &out {
                    for b in &next {
                        *work += 1;
                        if *work > 65536 {
                            return None;
                        }
                        let mut merged = a.clone();
                        let mut contradiction = false;
                        merged.partial |= b.partial;
                        for (c, v) in &b.equalities {
                            if merged
                                .equalities
                                .insert(c.clone(), *v)
                                .is_some_and(|old| old != *v)
                            {
                                contradiction = true;
                                break;
                            }
                        }
                        if !contradiction {
                            joined.push(merged);
                        }
                        if joined.len() > 256 {
                            return None;
                        }
                    }
                }
                out = joined;
            }
            out
        }
        Predicate::In(_, vs) if vs.is_empty() => vec![],
        Predicate::Compare(_, _, Value::Null(_)) => vec![],
        Predicate::Not(p) if matches!(p.as_ref(), Predicate::Compare(_, _, Value::Null(_))) => {
            vec![]
        }
        Predicate::Compare(c, Compare::Eq, v) if signed_column(c) && signed(v).is_some() => {
            vec![Branch {
                equalities: [(c.name.clone(), signed(v).unwrap())].into(),
                partial: false,
            }]
        }
        // Multiple IN and singleton IN deliberately do not establish key equality.
        Predicate::Compare(c, _, v) if signed_column(c) && signed(v).is_some() => {
            vec![Branch::default()]
        }
        Predicate::In(c, vs) if signed_column(c) && vs.iter().all(|v| signed(v).is_some()) => {
            vec![Branch::default()]
        }
        _ => unknown(),
    };
    Some(result)
}
fn predicate_columns(p: &Predicate, out: &mut Vec<Column>) {
    match p {
        Predicate::And(ps) | Predicate::Or(ps) => {
            for p in ps {
                predicate_columns(p, out);
            }
        }
        Predicate::Not(p) => predicate_columns(p, out),
        Predicate::Compare(c, _, _) | Predicate::In(c, _) | Predicate::IsNull(c) => {
            out.push(c.clone())
        }
    }
}
fn proof(
    record: &Record,
    facts: Option<&ScopeFacts>,
) -> (Scope, Completeness, Assumptions, Evidence) {
    use {Assumptions as A, Completeness as C, Evidence as E, Scope as S};
    let fail = |e| (S::Unknown, C::Unknown, A::None, e);
    let Some(f) = facts else {
        return fail(E::MissingFacts);
    };
    let d = &record.metadata.operation.0;
    let mut columns = vec![];
    match &d.body {
        Body::Select(ps) => columns.extend(ps.iter().map(|p| p.column.clone())),
        Body::Update(data) | Body::Insert(data) => {
            columns.extend(data.iter().map(|(c, _)| c.clone()))
        }
        _ => (),
    }
    columns.extend(d.order.iter().map(|o| o.column.clone()));
    if let Some(p) = &d.predicate {
        predicate_columns(p, &mut columns);
    }
    if f.table != d.table
        || f.dialect != record.dialect
        || columns.iter().any(|c| !f.columns.contains(c))
    {
        return fail(E::InconsistentFacts);
    }
    if f.context != TableContext::OrdinaryBaseTableWithoutEffects {
        return fail(E::UnsupportedContext);
    }
    if matches!(d.body, Body::Insert(_)) {
        return (
            S::NotApplicable,
            C::Unknown,
            A::ApplicationAsserted,
            E::InsertShape,
        );
    }
    let Some(p) = &d.predicate else {
        return (
            S::Broad,
            C::Complete,
            A::ApplicationAsserted,
            E::NoPredicate,
        );
    };
    let Some(bs) = branches(p, &mut 0) else {
        return fail(E::AnalysisBudget);
    };
    let a = A::ApplicationAsserted;
    if bs.is_empty() {
        return (S::Zero, C::Complete, a, E::EmptyTruthSet);
    }
    let c = if bs.iter().any(|b| b.partial) {
        C::Partial
    } else {
        C::Complete
    };
    let mut bound = None;
    for key in &f.keys {
        let cs = key.columns();
        if !cs.iter().all(signed_column) {
            continue;
        }
        let tuples: Option<Vec<Vec<i64>>> = bs
            .iter()
            .map(|b| {
                cs.iter()
                    .map(|c| b.equalities.get(&c.name).copied())
                    .collect()
            })
            .collect();
        if let Some(mut ts) = tuples {
            ts.sort();
            ts.dedup();
            bound = Some(bound.map_or(ts.len(), |old: usize| old.min(ts.len())));
        }
    }
    if let Some(n) = bound {
        return (
            if n == 1 {
                S::Single
            } else {
                S::FiniteBound(n as u16)
            },
            c,
            a,
            E::CompleteKeyTuples,
        );
    }
    if f.keys.is_empty()
        || f.keys
            .iter()
            .any(|k| !k.columns().iter().all(signed_column))
    {
        return (S::Unknown, C::Unknown, a, E::UnsupportedKey);
    }
    if c == C::Partial {
        (S::Unknown, c, a, E::UnsupportedPredicate)
    } else {
        (S::Broad, c, a, E::UnboundedPredicate)
    }
}
impl Plan {
    /// Inspect the captured semantics; no finalize/render/provider/DB call occurs.
    pub fn inspect(&self, facts: Option<&ScopeFacts>) -> Inspection<'_> {
        let (scope, completeness, assumptions, evidence) = proof(&self.record, facts);
        let d = &self.record.metadata.operation.0;
        let mut findings = vec![];
        let mut add = |code| findings.push(Finding { code });
        let write = matches!(d.body, Body::Update(_) | Body::Delete);
        let risk = if write && d.predicate.is_none() {
            add(FindingCode::MissingWhere);
            Risk::Blocked
        } else if matches!(d.body, Body::Insert(_)) {
            add(FindingCode::InsertUnverified);
            Risk::High
        } else {
            if scope == Scope::Unknown || completeness != Completeness::Complete {
                add(FindingCode::UnknownProof);
            }
            if write {
                match scope {
                    Scope::Zero | Scope::Single if completeness == Completeness::Complete => {
                        Risk::Low
                    }
                    Scope::Zero | Scope::Single => Risk::High,
                    Scope::FiniteBound(_) => {
                        add(FindingCode::BulkWrite);
                        Risk::High
                    }
                    Scope::Broad => {
                        add(FindingCode::WeakPredicate);
                        if matches!(d.body, Body::Delete) {
                            Risk::Destructive
                        } else {
                            Risk::High
                        }
                    }
                    _ => Risk::High,
                }
            } else {
                if matches!(d.body, Body::Select(_)) {
                    add(FindingCode::ReadProjection);
                }
                if d.limit.is_none() {
                    add(FindingCode::ReadWithoutLimit);
                }
                if scope == Scope::Unknown || completeness != Completeness::Complete {
                    Risk::High
                } else if d.limit.is_none() {
                    Risk::Medium
                } else {
                    Risk::Low
                }
            }
        };
        Inspection {
            record: &self.record,
            scope,
            completeness,
            assumptions,
            evidence,
            risk,
            findings,
        }
    }
}
