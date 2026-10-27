//! DB-free finite policy preparation; never an authorization or execution gate.
use super::*;

/// Fixed, input-free policy failures. No arbitrary error/source chain is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    InvalidPlan(PlanError),
    MissingSchema,
    Identity,
    Coverage,
    Unsupported,
    MissingTenant,
    TenantMismatch,
    TenantPredicate,
    TenantWrite,
    PiiAccess,
    RequiredFilter,
    ProtectedWrite,
    Budget,
}
impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_POLICY_{self:?}")
    }
}
impl std::error::Error for PolicyError {}
impl From<PlanError> for PolicyError {
    fn from(e: PlanError) -> Self {
        if e == PlanError::Budget {
            Self::Budget
        } else {
            Self::InvalidPlan(e)
        }
    }
}
type PolicyResult<T> = std::result::Result<T, PolicyError>;

/// Private immutable association of final record, original operation and exact
/// application snapshot. Success means preparation only, never permit or dispatch.
/// ```compile_fail
/// fn dispatch(p: quent_core::PolicyPreparation) { p.execute(); }
/// ```
/// ```compile_fail
/// fn alter(p: &mut quent_core::PolicyPreparation) { p.snapshot = None; }
/// ```
#[derive(PartialEq, Eq)]
pub struct PolicyPreparation {
    plan: Plan,
    original: Operation,
    snapshot: DatabaseContext,
    fact_index: usize,
}
impl PolicyPreparation {
    /// Inspect the final record using only the retained exact application facts.
    pub fn inspect(&self) -> Inspection<'_> {
        self.plan.inspect(Some(self.facts()))
    }
    fn facts(&self) -> &ScopeFacts {
        &self.snapshot.application.schema.as_ref().unwrap().facts[self.fact_index]
    }
    /// Explicit local sensitive boundary, not a public summary or execution handle.
    pub fn sensitive_source(&self) -> SensitivePolicySource<'_> {
        SensitivePolicySource { prepared: self }
    }
}
pub struct SensitivePolicySource<'a> {
    prepared: &'a PolicyPreparation,
}
impl SensitivePolicySource<'_> {
    pub fn final_source(&self) -> SensitiveSource<'_> {
        self.prepared.plan.sensitive_source()
    }
    pub fn original_operation(&self) -> &Operation {
        &self.prepared.original
    }
    pub fn context(&self) -> &DatabaseContext {
        &self.prepared.snapshot
    }
}
redacted!(PolicyPreparation, SensitivePolicySource<'_>);

// Common finite physical-identity spelling domain. In particular, MySQL column
// case folding must not make two nominal Rust columns alias the tenant column.
fn canonical(id: &Ident) -> bool {
    id.0.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
fn canonical_table(t: &Table) -> bool {
    t.alias.is_none() && t.path.len() == 2 && t.path.iter().all(canonical)
}
fn columns(d: &Draft) -> Vec<&Column> {
    let mut out = vec![];
    match &d.body {
        Body::Select(ps) => out.extend(ps.iter().map(|p| &p.column)),
        Body::Insert(ds) | Body::Update(ds) => out.extend(ds.iter().map(|(c, _)| c)),
        _ => (),
    }
    out.extend(d.returning.iter().map(|p| &p.column));
    out.extend(d.version.iter().map(|(c, _)| c));
    out.extend(d.order.iter().map(|o| &o.column));
    fn visit<'a>(p: &'a Predicate, out: &mut Vec<&'a Column>) {
        match p {
            Predicate::And(ps) | Predicate::Or(ps) => {
                for p in ps {
                    visit(p, out);
                }
            }
            Predicate::Not(p) => visit(p, out),
            Predicate::Compare(c, _, _) | Predicate::In(c, _) | Predicate::IsNull(c) => out.push(c),
        }
    }
    if let Some(p) = &d.predicate {
        visit(p, &mut out);
    }
    out
}

// Check *every occurrence*, even in dead branches. NOT/ranges/NULL are never
// upgraded to tenant equality by the appended guard. No manual-proof mode exists.
fn tenant_occurrences(p: &Predicate, tenant: &TenantSnapshot, negated: bool) -> PolicyResult<()> {
    match p {
        Predicate::And(ps) | Predicate::Or(ps) => {
            for p in ps {
                tenant_occurrences(p, tenant, negated)?;
            }
        }
        Predicate::Not(p) => tenant_occurrences(p, tenant, true)?,
        Predicate::Compare(c, op, v) if c == &tenant.column => {
            if negated || *op != Compare::Eq {
                return Err(PolicyError::TenantPredicate);
            }
            if v != &tenant.value {
                return Err(PolicyError::TenantMismatch);
            }
        }
        Predicate::In(c, vs) if c == &tenant.column => {
            if negated {
                return Err(PolicyError::TenantPredicate);
            }
            for v in vs {
                if v != &tenant.value {
                    return Err(PolicyError::TenantMismatch);
                }
            }
        }
        Predicate::IsNull(c) if c == &tenant.column => return Err(PolicyError::TenantPredicate),
        _ => (),
    }
    Ok(())
}
// RequiredFilter is deliberately narrower than general scope proof: a supported
// signed non-NULL equality in every syntactic OR arm. No NOT, NULL, range or
// occurrence-only proof; false/contradictory arms do not bypass this requirement.
fn required(p: &Predicate, column: &Column) -> bool {
    match p {
        Predicate::And(ps) => ps.iter().any(|p| required(p, column)),
        Predicate::Or(ps) => !ps.is_empty() && ps.iter().all(|p| required(p, column)),
        Predicate::Compare(c, Compare::Eq, v) => {
            c == column
                && !c.nullable
                && c.sql_type == v.sql_type()
                && matches!(
                    v,
                    Value::I8(_) | Value::I16(_) | Value::I32(_) | Value::I64(_)
                )
        }
        _ => false,
    }
}
fn append(d: &mut Draft, guards: Vec<Predicate>) {
    if guards.is_empty() {
        return;
    }
    let mut all = Vec::new();
    if let Some(caller) = d.predicate.take() {
        all.push(caller);
    }
    all.extend(guards);
    d.predicate = Some(Predicate::And(all));
}
impl PreparationContext {
    /// All five supported IR families share this application-only path. Exact
    /// qualified identity and complete asserted ordinary-table coverage required.
    /// No connection/provider/clock/authorization/transaction is consulted.
    pub fn prepare_operation(&self, operation: &Operation) -> PolicyResult<PolicyPreparation> {
        // Before any recursive clone or traversal. Raw finalize uses this too.
        operation.check_budget()?;
        let snapshot = &self.snapshot;
        let binding = &snapshot.settings.binding;
        let d = &operation.0;
        if !canonical_table(&d.table)
            || snapshot
                .settings
                .policies
                .declarations
                .iter()
                .any(|p| !canonical_table(&p.column().table) || !canonical(&p.column().name))
        {
            return Err(PolicyError::Unsupported);
        }
        if binding.dialect == Dialect::Mysql && d.table.path[0] != binding.catalog {
            return Err(PolicyError::Identity);
        }
        let schema = snapshot
            .application
            .schema
            .as_ref()
            .ok_or(PolicyError::MissingSchema)?;
        let fact_index = schema
            .facts
            .iter()
            .position(|f| f.table == d.table)
            .ok_or(PolicyError::Coverage)?;
        let facts = &schema.facts[fact_index];
        if facts.columns.iter().any(|c| !canonical(&c.name))
            || facts.context != TableContext::OrdinaryBaseTableWithoutEffects
        {
            return Err(PolicyError::Unsupported);
        }
        let used = columns(d);
        if used.iter().any(|c| !facts.columns.contains(c)) {
            return Err(PolicyError::Coverage);
        }
        let declarations = &snapshot.settings.policies.declarations;
        // No fallback from an alternate schema/path/alias spelling to no policy.
        if declarations.iter().any(|p| {
            let t = &p.column().table;
            t.path.last() == d.table.path.last() && t != &d.table
        }) {
            return Err(PolicyError::Identity);
        }
        let applicable: Vec<_> = declarations
            .iter()
            .filter(|p| p.column().table == d.table)
            .collect();
        for p in &applicable {
            if !facts.columns.contains(p.column()) {
                return Err(PolicyError::Coverage);
            }
        }
        if let Some(p) = &d.predicate
            && !inspection::within_branch_budget(p)
        {
            return Err(PolicyError::Budget);
        }
        let mut guards = vec![];
        // All inspection precedes cloning; final rendering validates all native
        // values/columns/shapes with the existing sole renderer.
        for policy in &applicable {
            let c = policy.column();
            match policy {
                PolicyDeclaration::Tenant(_) => {
                    let tenant = snapshot
                        .application
                        .tenant
                        .as_ref()
                        .ok_or(PolicyError::MissingTenant)?;
                    if tenant.column != *c || tenant.binding != *binding {
                        return Err(PolicyError::Identity);
                    }
                    if let Some(p) = &d.predicate {
                        tenant_occurrences(p, tenant, false)?;
                    }
                    match &d.body {
                        Body::Update(ds)
                            if ds.iter().any(|(column, _)| column == c)
                                || d.version.as_ref().is_some_and(|(column, _)| column == c) =>
                        {
                            return Err(PolicyError::TenantWrite);
                        }
                        Body::Insert(ds) => {
                            for (column, input) in ds {
                                if column == c && input != &Input::Value(tenant.value.clone()) {
                                    return Err(PolicyError::TenantMismatch);
                                }
                            }
                        }
                        _ => (),
                    }
                    if !matches!(d.body, Body::Insert(_)) {
                        guards.push(Predicate::Compare(
                            c.clone(),
                            Compare::Eq,
                            tenant.value.clone(),
                        ));
                    }
                }
                PolicyDeclaration::SoftDelete(_) if !matches!(d.body, Body::Insert(_)) => {
                    guards.push(Predicate::Compare(
                        c.clone(),
                        Compare::Eq,
                        Value::Bool(false),
                    ));
                }
                PolicyDeclaration::Pii(_) => {
                    // DELETE affects the whole row, including unprojected PII.
                    if matches!(d.body, Body::Delete) || used.contains(&c) {
                        return Err(PolicyError::PiiAccess);
                    }
                }
                PolicyDeclaration::RequiredFilter(_) => {
                    if matches!(d.body, Body::Insert(_))
                        || !d.predicate.as_ref().is_some_and(|p| required(p, c))
                    {
                        return Err(PolicyError::RequiredFilter);
                    }
                }
                PolicyDeclaration::Readonly(_) | PolicyDeclaration::Generated(_) => {
                    if let Body::Update(ds) | Body::Insert(ds) = &d.body
                        && (ds.iter().any(|(column, _)| column == c)
                            || d.version.as_ref().is_some_and(|(column, _)| column == c))
                    {
                        return Err(PolicyError::ProtectedWrite);
                    }
                }
                _ => (),
            }
        }
        let mut final_operation = operation.clone();
        if let Body::Insert(ds) = &mut final_operation.0.body {
            for p in &applicable {
                if let PolicyDeclaration::Tenant(c) = p
                    && !ds.iter().any(|(column, _)| column == c)
                {
                    ds.push((
                        c.clone(),
                        Input::Value(snapshot.application.tenant.as_ref().unwrap().value.clone()),
                    ));
                }
            }
        } else {
            append(&mut final_operation.0, guards);
        }
        // Added depth/nodes/args/columns must fit the same budgets as raw plans.
        final_operation.check_budget()?;
        if let Some(p) = &final_operation.0.predicate
            && !inspection::within_branch_budget(p)
        {
            return Err(PolicyError::Budget);
        }
        let mut plan = final_operation.finalize(binding.dialect)?;
        plan.record.caller_missing_where =
            matches!(d.body, Body::Update(_) | Body::Delete) && d.predicate.is_none();
        Ok(PolicyPreparation {
            plan,
            original: operation.clone(),
            snapshot: snapshot.clone(),
            fact_index,
        })
    }
}

impl PreparationContext {
    /// DB-free finite command preparation. Not an execution permit. INSERT must
    /// explicitly cover the complete asserted schema after tenant supplementation.
    /// Ordinary preparation and raw INSERT inspection remain unverified/High.
    pub fn prepare_command_operation(
        &self,
        operation: &Operation,
    ) -> PolicyResult<PolicyPreparation> {
        let mut p = self.prepare_operation(operation)?;
        let d = &p.plan.record.metadata.operation.0;
        let tenant = p
            .snapshot
            .application
            .tenant
            .as_ref()
            .ok_or(PolicyError::MissingTenant)?;
        if tenant.column.table != d.table
            || !p
                .snapshot
                .settings
                .policies
                .declarations
                .iter()
                .any(|p| matches!(p,PolicyDeclaration::Tenant(c) if c == &tenant.column))
        {
            return Err(PolicyError::Unsupported);
        }
        if !d.returning.is_empty() {
            return Err(PolicyError::Unsupported);
        }
        match &d.body {
            Body::Insert(data) => {
                let columns = &p.facts().columns;
                if data.len() != columns.len()
                    || !columns
                        .iter()
                        .all(|c| data.iter().filter(|(supplied, _)| supplied == c).count() == 1)
                {
                    return Err(PolicyError::Unsupported);
                }
                p.plan.record.insert_proof = Some(FiniteInsertProof::AllAssertedColumns);
            }
            Body::Update(_) => (),
            _ => return Err(PolicyError::Unsupported),
        }
        Ok(p)
    }
}
impl SensitivePolicySource<'_> {
    /// Sensitive, versioned deterministic encoding of the original operation,
    /// complete context and final immutable record. No decoder/restoration exists.
    pub fn canonical_command_input(&self, ledger: &Table) -> PolicyResult<CommandCanonical> {
        use crate::plan::command_encoding::Encoder;
        // QCC1 fixes exact-one in the protocol; version is already retained in
        // BOTH original/final operations and the sole renderer's native args.
        let mut e = Encoder::new(b"QCC1");
        e.table(ledger)?;
        e.operation(&self.prepared.original)?;
        self.prepared.snapshot.encode_command_context(&mut e)?;
        let r = &self.prepared.plan.record;
        e.operation(&r.metadata.operation)?;
        e.bytes(r.sql.as_bytes())?;
        e.values(&r.args)?;
        e.boolean(r.caller_missing_where)?;
        e.boolean(r.insert_proof.is_some())?;
        Ok(CommandCanonical(e.finish()))
    }
    /// Durable lookup key: target (including catalog/dialect/routing), tenant,
    /// namespace and application-persisted operation ID. Subject is in full input,
    /// so another subject cannot disclose a stored result under this key.
    pub fn canonical_command_key(
        &self,
        namespace: &[u8],
        operation_id: &[u8; 16],
    ) -> PolicyResult<CommandCanonical> {
        use crate::plan::command_encoding::Encoder;
        if namespace.is_empty() || namespace.len() > 128 {
            return Err(PolicyError::Budget);
        }
        let mut e = Encoder::new(b"QCK1");
        self.prepared
            .snapshot
            .encode_command_target_tenant(&mut e)?;
        e.bytes(namespace)?;
        e.bytes(operation_id)?;
        let bytes = e.finish();
        if bytes.len() > 2048 {
            return Err(PolicyError::Budget);
        }
        Ok(CommandCanonical(bytes))
    }
}
