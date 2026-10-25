//! Pure offline review. Independent data input, serialize-only public output.
//! No output can reconstruct a plan, mapping, context, permission or handle.
use crate::{manifest::*, operation_spec::*};
use quent_core::{Completeness, DatabaseContext, Inspection, Risk};
use serde::Serialize;
use std::fmt;
mod plan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Unknown,
    Partial,
    Unsupported,
    Refused,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    SourceParsed,
    Incomplete,
    Unsupported,
    Invalid,
    Budget,
    StructuralBlock,
    ApplicationInspection,
}
/// All axes are independent. A checked wire is never a checked policy/runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Precision {
    wire: &'static str,
    sql: &'static str,
    native: &'static str,
    model: &'static str,
    binding: &'static str,
    policy: &'static str,
    tenant: &'static str,
    freshness: &'static str,
    runtime: &'static str,
    external_verdict: &'static str,
}
impl Default for Precision {
    fn default() -> Self {
        Self {
            wire: "checked",
            sql: "unverified",
            native: "unverified",
            model: "unverified",
            binding: "unverified",
            policy: "unverified",
            tenant: "unverified",
            freshness: "unverified",
            runtime: "unverified",
            external_verdict: "unverified",
        }
    }
}
/// Private construction and Serialize only; no general input or authority API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Review {
    kind: &'static str,
    version: u8,
    source: &'static str,
    state: State,
    code: Code,
    risk: &'static str,
    coverage: &'static str,
    precision: Precision,
    action: &'static str,
    // Constructed exclusively from existing fixed output serializers, never input JSON.
    inspection: Option<serde_json::Value>,
    spec: Option<SpecSummary>,
    manifest: Option<serde_json::Value>,
    claims: &'static str,
    claim_count: u8,
}
impl Review {
    fn new(source: &'static str) -> Self {
        Self {
            kind: "quent_review",
            version: 1,
            source,
            state: State::Unknown,
            code: Code::SourceParsed,
            risk: "unknown",
            coverage: "unknown",
            precision: Precision::default(),
            action: "supply_application_context",
            inspection: None,
            spec: None,
            manifest: None,
            claims: "absent",
            claim_count: 0,
        }
    }
    pub fn state(&self) -> State {
        self.state
    }
    /// No exit 0 exists: even valid input has unverified runtime/authority axes.
    pub fn exit_code(&self) -> u8 {
        match self.state {
            State::Unknown | State::Partial => 3,
            State::Unsupported => 4,
            State::Refused => 5,
        }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("fixed review output")
    }
    /// Pretty output contains exactly the same closed fields as public JSON.
    pub fn to_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("fixed review output")
    }
}
impl fmt::Display for Review {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewError {
    Invalid,
    Budget,
    Unsupported,
}
/// Versioned output-only diagnostic. It retains no parser/OS/source error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ReviewDiagnostic {
    kind: &'static str,
    version: u8,
    state: &'static str,
    code: ReviewError,
    action: &'static str,
}
impl ReviewDiagnostic {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("fixed review diagnostic")
    }
}
impl ReviewError {
    pub fn public_view(self) -> ReviewDiagnostic {
        ReviewDiagnostic {
            kind: "quent_review_diagnostic",
            version: 1,
            state: "refused",
            code: self,
            action: "check_supported_input",
        }
    }
    pub fn exit_code(self) -> u8 {
        if self == Self::Unsupported { 4 } else { 2 }
    }
    pub fn code(self) -> &'static str {
        match self {
            Self::Invalid => "QUENT_REVIEW_INVALID",
            Self::Budget => "QUENT_REVIEW_BUDGET",
            Self::Unsupported => "QUENT_REVIEW_UNSUPPORTED",
        }
    }
}
impl fmt::Display for ReviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}
impl std::error::Error for ReviewError {}
impl From<SpecError> for ReviewError {
    fn from(e: SpecError) -> Self {
        match e {
            SpecError::Budget => Self::Budget,
            SpecError::Unsupported => Self::Unsupported,
            _ => Self::Invalid,
        }
    }
}
/// External finite data only; temporary Core objects never leave this function.
pub fn plan_source(bytes: &[u8]) -> Result<Review, ReviewError> {
    plan::review(bytes)
}
/// Syntax/range checking only. The CLI has no trusted mapping/current context.
pub fn operation_spec(bytes: &[u8]) -> Result<Review, ReviewError> {
    let source = SpecSource::parse(bytes)?;
    let mut r = Review::new("operation_spec");
    r.precision.native = "literal_ranges_checked";
    r.spec = Some(source.public_view());
    Ok(r)
}
/// Explicit app mapping/current reuse the original exact-content common compiler.
pub fn typed_spec(
    source: SpecSource,
    mapping: &TypedModelContract,
    current: &DatabaseContext,
) -> Result<Review, TypedError> {
    let compiled = source.compile(mapping, current)?;
    let mut r = inspection(&compiled.inspect());
    r.source = "application_typed_spec";
    r.precision.wire = "checked";
    r.precision.model = "application_asserted_checked";
    r.precision.native = "application_asserted_checked";
    r.precision.binding = "exact_application_snapshot";
    r.precision.policy = "application_asserted_checked";
    r.precision.tenant = "application_asserted_checked";
    Ok(r)
}
/// Projects the existing inspection's SAME record. No rendering or SQL parsing.
pub fn inspection(i: &Inspection<'_>) -> Review {
    let mut r = Review::new("application_inspection");
    r.code = Code::ApplicationInspection;
    r.precision.wire = "not_applicable";
    r.precision.external_verdict = "not_applicable";
    r.risk = i.public_view().risk.code();
    r.coverage = match i.completeness() {
        Completeness::Complete => "complete",
        Completeness::Partial => "partial",
        Completeness::Unknown => "unknown",
    };
    r.precision.sql = "retained_record";
    // Existing output contains full scope, evidence and all nonsuppressible findings.
    r.inspection =
        Some(serde_json::from_str(&i.public_view().to_json()).expect("fixed core output"));
    r.state = if matches!(i.risk(), Risk::High | Risk::Destructive | Risk::Blocked) {
        State::Refused
    } else if i.completeness() == Completeness::Partial {
        State::Partial
    } else {
        State::Unknown
    };
    if i.risk() == Risk::Blocked {
        r.code = Code::StructuralBlock;
    }
    r
}
/// Uses existing independent schema/policy/code/live source coverage. No clock,
/// live workflow, assertion constructor or fingerprint is consulted.
pub fn manifest(bytes: &[u8]) -> Result<Review, ReviewError> {
    let source = ManifestSource::parse(bytes).map_err(|e| match e {
        ManifestError::Budget => ReviewError::Budget,
        ManifestError::Version => ReviewError::Unsupported,
        _ => ReviewError::Invalid,
    })?;
    let summary = source.public_view();
    let mut r = Review::new("manifest");
    r.action = "review_independent_domains";
    if summary
        .artifacts()
        .iter()
        .any(|a| a.coverage() == SourceCoverage::Unsupported)
    {
        r.state = State::Unsupported;
        r.code = Code::Unsupported;
    } else if summary
        .artifacts()
        .iter()
        .any(|a| a.coverage() == SourceCoverage::Partial)
    {
        r.state = State::Partial;
        r.code = Code::Incomplete;
    }
    r.manifest = Some(serde_json::from_str(&summary.to_json()).expect("fixed manifest output"));
    Ok(r)
}
