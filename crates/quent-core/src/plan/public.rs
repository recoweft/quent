//! Output-only finite summaries. Never execution authority or restoration input.
use super::*;

// Every string emitted by this module is a literal in this closed register.
macro_rules! register {
    ($name:ident { $($variant:ident => $wire:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub const fn code(self) -> &'static str {
                match self { $(Self::$variant => $wire),+ }
            }
        }
    };
}
register!(ViewRisk { Unevaluated => "unevaluated", Low => "low", Medium => "medium", High => "high", Destructive => "destructive", Blocked => "blocked" });
register!(ViewPrecision { Unevaluated => "unevaluated", ConditionalConservative => "conditional_conservative", Unknown => "unknown" });
register!(ViewState { Observed => "observed", Uncommitted => "uncommitted", Committed => "committed", NotStarted => "not_started", RolledBack => "rolled_back", OutcomeUnknown => "outcome_unknown" });
register!(DiagnosticCode {
    PlanIdentifier => "plan_identifier",
    PlanShape => "plan_shape",
    PlanType => "plan_type",
    PlanMissing => "plan_missing",
    PlanUnsupported => "plan_unsupported",
    PlanBudget => "plan_budget",
    ContextIdentity => "context_identity",
    ContextGeneration => "context_generation",
    ContextBinding => "context_binding",
    ContextPolicy => "context_policy",
    ContextType => "context_type",
    ContextUnsupported => "context_unsupported",
    ContextBudget => "context_budget",
    PolicyMissingSchema => "policy_missing_schema",
    PolicyIdentity => "policy_identity",
    PolicyCoverage => "policy_coverage",
    PolicyUnsupported => "policy_unsupported",
    PolicyMissingTenant => "policy_missing_tenant",
    PolicyTenantMismatch => "policy_tenant_mismatch",
    PolicyTenantPredicate => "policy_tenant_predicate",
    PolicyTenantWrite => "policy_tenant_write",
    PolicyPiiAccess => "policy_pii_access",
    PolicyRequiredFilter => "policy_required_filter",
    PolicyProtectedWrite => "policy_protected_write",
    PolicyBudget => "policy_budget",
    ExecutionContextMismatch => "execution_context_mismatch",
    ExecutionOperationMismatch => "execution_operation_mismatch",
    ExecutionInvalidTtl => "execution_invalid_ttl",
    ExecutionExpired => "execution_expired",
    ExecutionMissingSubject => "execution_missing_subject",
    ExecutionRiskRefused => "execution_risk_refused",
    ExecutionIncomplete => "execution_incomplete",
    ExecutionUnsupported => "execution_unsupported",
    ExecutionEncode => "execution_encode",
    ExecutionConnect => "execution_connect",
    ExecutionReadFailed => "execution_read_failed",
    ExecutionDecode => "execution_decode",
    ExecutionResultLimit => "execution_result_limit",
    ExecutionOutcomeUnknown => "execution_outcome_unknown",
    ExecutionSessionUnavailable => "execution_session_unavailable",
    ExecutionExpectationMismatch => "execution_expectation_mismatch",
    ExecutionConflict => "execution_conflict",
});
/// Fixed code plus an optional finite atomic disposition. No error object retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticView {
    pub code: DiagnosticCode,
    pub state: Option<ViewState>,
}
impl DiagnosticView {
    pub fn to_json(&self) -> String {
        let state = self
            .state
            .map_or("null".into(), |s| format!("\"{}\"", s.code()));
        format!(
            "{{\"kind\":\"quent_diagnostic_view\",\"version\":1,\"code\":\"{}\",\"state\":{state}}}",
            self.code.code()
        )
    }
}
impl fmt::Display for DiagnosticView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
impl std::error::Error for DiagnosticView {}
impl PlanError {
    pub fn public_view(&self) -> DiagnosticView {
        let code = match self {
            Self::Identifier => DiagnosticCode::PlanIdentifier,
            Self::Shape => DiagnosticCode::PlanShape,
            Self::Type => DiagnosticCode::PlanType,
            Self::Missing => DiagnosticCode::PlanMissing,
            Self::Unsupported => DiagnosticCode::PlanUnsupported,
            Self::Budget => DiagnosticCode::PlanBudget,
        };
        DiagnosticView { code, state: None }
    }
}
impl ContextError {
    pub fn public_view(&self) -> DiagnosticView {
        let code = match self {
            Self::Identity => DiagnosticCode::ContextIdentity,
            Self::Generation => DiagnosticCode::ContextGeneration,
            Self::Binding => DiagnosticCode::ContextBinding,
            Self::Policy => DiagnosticCode::ContextPolicy,
            Self::Type => DiagnosticCode::ContextType,
            Self::Unsupported => DiagnosticCode::ContextUnsupported,
            Self::Budget => DiagnosticCode::ContextBudget,
        };
        DiagnosticView { code, state: None }
    }
}
impl PolicyError {
    pub fn public_view(&self) -> DiagnosticView {
        let code = match self {
            Self::InvalidPlan(e) => return e.public_view(),
            Self::MissingSchema => DiagnosticCode::PolicyMissingSchema,
            Self::Identity => DiagnosticCode::PolicyIdentity,
            Self::Coverage => DiagnosticCode::PolicyCoverage,
            Self::Unsupported => DiagnosticCode::PolicyUnsupported,
            Self::MissingTenant => DiagnosticCode::PolicyMissingTenant,
            Self::TenantMismatch => DiagnosticCode::PolicyTenantMismatch,
            Self::TenantPredicate => DiagnosticCode::PolicyTenantPredicate,
            Self::TenantWrite => DiagnosticCode::PolicyTenantWrite,
            Self::PiiAccess => DiagnosticCode::PolicyPiiAccess,
            Self::RequiredFilter => DiagnosticCode::PolicyRequiredFilter,
            Self::ProtectedWrite => DiagnosticCode::PolicyProtectedWrite,
            Self::Budget => DiagnosticCode::PolicyBudget,
        };
        DiagnosticView { code, state: None }
    }
}
/// Pure display copy. Bounds describe target scope under application assertions,
/// never actual cardinality, live verification, authorization or transaction effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanView {
    pub risk: ViewRisk,
    pub precision: ViewPrecision,
    pub scope: Scope,
    pub completeness: Completeness,
    pub assumptions: Assumptions,
    pub evidence: Option<Evidence>,
    // Private bounded findings prevent caller-provided unbounded lists.
    findings: Vec<FindingCode>,
    family: &'static str,
    projection: &'static str,
    projection_count: u16,
    write_count: u16,
    order_count: u16,
    versioned: bool,
    returning_count: u16,
    caller_missing_where: bool,
    has_limit: bool,
}
impl PlanView {
    pub fn findings(&self) -> &[FindingCode] {
        &self.findings
    }
    fn record(record: &Record) -> Self {
        let m = &record.metadata;
        let d = &m.operation.0;
        let (projection, projection_count) = match &m.projection {
            FinalProjection::Columns(c) => ("columns", c.len() as u16),
            FinalProjection::Count => ("count", 1),
            FinalProjection::None => ("none", 0),
        };
        Self {
            risk: ViewRisk::Unevaluated,
            precision: ViewPrecision::Unevaluated,
            scope: Scope::Unknown,
            completeness: Completeness::Unknown,
            assumptions: Assumptions::None,
            evidence: None,
            findings: vec![],
            family: match m.family {
                Family::Select => "select",
                Family::Count => "count",
                Family::Insert => "insert",
                Family::Update => "update",
                Family::Delete => "delete",
            },
            projection,
            projection_count,
            write_count: m.write_data.len() as u16,
            order_count: d.order.len() as u16,
            versioned: d.version.is_some(),
            returning_count: d.returning.len() as u16,
            caller_missing_where: record.caller_missing_where,
            has_limit: d.limit.is_some(),
        }
    }
    /// Stable allowlist JSON. No Debug, source string, metadata clone or serializer
    /// callback participates. Enum spellings are explicitly registered below.
    pub fn to_json(&self) -> String {
        let (scope, upper) = match self.scope {
            Scope::Zero => ("zero", "0".into()),
            Scope::Single => ("single", "1".into()),
            Scope::FiniteBound(n) => ("finite_bound", n.to_string()),
            Scope::Broad => ("broad", "null".into()),
            Scope::Unknown => ("unknown", "null".into()),
            Scope::NotApplicable => ("not_applicable", "null".into()),
        };
        let completeness = match self.completeness {
            Completeness::Complete => "complete",
            Completeness::Partial => "partial",
            Completeness::Unknown => "unknown",
        };
        let assumptions = match self.assumptions {
            Assumptions::None => "none",
            Assumptions::ApplicationAsserted => "application_asserted",
        };
        let evidence = self
            .evidence
            .map_or("null".into(), |e| format!("\"{}\"", evidence_code(e)));
        let findings = self
            .findings
            .iter()
            .map(|c| format!("\"{}\"", finding_code(*c)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            concat!(
                "{{\"kind\":\"quent_plan_view\",\"version\":1,",
                "\"precision\":\"{}\",\"risk\":\"{}\",\"scope\":\"{scope}\",\"scope_upper_bound\":{upper},",
                "\"completeness\":\"{completeness}\",\"assumptions\":\"{assumptions}\",",
                "\"evidence\":{evidence},\"findings\":[{findings}],\"family\":\"{}\",",
                "\"projection\":\"{}\",\"projection_count\":{},\"write_count\":{},",
                "\"order_count\":{},\"versioned\":{},\"returning_count\":{},",
                "\"caller_missing_where\":{},\"has_limit\":{}}}"
            ),
            self.precision.code(),
            self.risk.code(),
            self.family,
            self.projection,
            self.projection_count,
            self.write_count,
            self.order_count,
            self.versioned,
            self.returning_count,
            self.caller_missing_where,
            self.has_limit,
            scope = scope,
            upper = upper,
            completeness = completeness,
            assumptions = assumptions,
            evidence = evidence,
            findings = findings
        )
    }
}
impl fmt::Display for PlanView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
impl Plan {
    /// Uninspected raw record: risk unevaluated, completeness/scope unknown.
    pub fn public_view(&self) -> PlanView {
        PlanView::record(&self.record)
    }
}
impl Inspection<'_> {
    pub fn public_view(&self) -> PlanView {
        let mut v = PlanView::record(self.sensitive_source().record);
        v.scope = self.scope();
        v.precision = if matches!(v.scope, Scope::Unknown | Scope::NotApplicable) {
            ViewPrecision::Unknown
        } else {
            ViewPrecision::ConditionalConservative
        };
        v.completeness = self.completeness();
        v.assumptions = self.assumptions();
        v.evidence = Some(self.evidence());
        v.risk = match self.risk() {
            Risk::Low => ViewRisk::Low,
            Risk::Medium => ViewRisk::Medium,
            Risk::High => ViewRisk::High,
            Risk::Destructive => ViewRisk::Destructive,
            Risk::Blocked => ViewRisk::Blocked,
        };
        v.findings = self.findings().iter().map(Finding::code).collect();
        v
    }
}
impl PolicyPreparation {
    pub fn public_view(&self) -> PlanView {
        self.inspect().public_view()
    }
}
fn evidence_code(v: Evidence) -> &'static str {
    match v {
        Evidence::MissingFacts => "missing_facts",
        Evidence::InconsistentFacts => "inconsistent_facts",
        Evidence::UnsupportedContext => "unsupported_context",
        Evidence::UnsupportedKey => "unsupported_key",
        Evidence::AnalysisBudget => "analysis_budget",
        Evidence::EmptyTruthSet => "empty_truth_set",
        Evidence::CompleteKeyTuples => "complete_key_tuples",
        Evidence::UnboundedPredicate => "unbounded_predicate",
        Evidence::UnsupportedPredicate => "unsupported_predicate",
        Evidence::NoPredicate => "no_predicate",
        Evidence::InsertShape => "insert_shape",
    }
}
fn finding_code(v: FindingCode) -> &'static str {
    match v {
        FindingCode::MissingWhere => "missing_where",
        FindingCode::UnknownProof => "unknown_proof",
        FindingCode::BulkWrite => "bulk_write",
        FindingCode::WeakPredicate => "weak_predicate",
        FindingCode::ReadWithoutLimit => "read_without_limit",
        FindingCode::ReadProjection => "read_projection",
        FindingCode::InsertUnverified => "insert_unverified",
    }
}
