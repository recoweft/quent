//! Bounded presentation over full offline reviews. No DB, clock, Git, FS or permit.
//! Public output is never accepted as review input or repository coverage evidence.
//! ```compile_fail
//! let _: quent::review_ci::Report = serde_json::from_str("{}").unwrap();
//! ```
//! ```compile_fail
//! let _: quent_core::Plan = quent::review_ci::InputSet::parse(b"{}").unwrap().into();
//! ```
use crate::review::{self, ReviewError as Error};
use serde::{Deserialize, Serialize};
use std::fmt;

type Result<T> = std::result::Result<T, Error>;
pub const MAX_ENTRIES: usize = 8;
pub const MAX_DIAGNOSTICS: usize = 256;
pub const PUBLIC_DIAGNOSTICS: usize = 64;
pub const ANALYSIS_VERSION: u8 = 1;
const MAX_TIME: u64 = 253_402_300_799;
const MAX_LIFETIME: u64 = 31_536_000;

macro_rules! redacted {
    ($($t:ty),+) => {$ (impl fmt::Debug for $t {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("ReviewInput([REDACTED])") }
    })+};
}
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Format {
    Plan,
    Spec,
    Manifest,
    Rust,
}
#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    target: String,
    context: String,
    scope: String,
    format: Format,
    source: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetWire {
    kind: String,
    version: u8,
    analysis_version: u8,
    selection: Selection,
    inputs: Vec<Input>,
}
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Selection {
    ExplicitSet,
    ChangedOnly,
}
/// Sensitive closed input, never a Plan/context/mapping or a complete-head claim.
pub struct InputSet {
    selection: Selection,
    records: Vec<Record>,
}
struct Record {
    input: Option<Input>,
    full: Full,
    diagnostics: Vec<Code>,
}
#[derive(Serialize)]
#[serde(untagged)]
enum Full {
    Review(review::Review),
    #[cfg(feature = "source-review")]
    Rust(crate::source_review::Review),
}
impl Full {
    fn exit(&self) -> u8 {
        match self {
            Self::Review(r) => r.exit_code(),
            #[cfg(feature = "source-review")]
            Self::Rust(r) => r.exit_code(),
        }
    }
}
/// Fixed messages are the ONLY SARIF message source. No general Diagnostic/Error
/// serializer or caller-provided rule/property/name is accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    ReviewIncomplete,
    StructuralBlock,
    Unsupported,
    InputBudget,
    MissingWhere,
    UnknownProof,
    BulkWrite,
    WeakPredicate,
    ReadWithoutLimit,
    ReadProjection,
    InsertUnverified,
    IdentityUnverified,
    ContextUnverified,
    NoCandidates,
    MacroUnexpanded,
    AttributeUnevaluated,
    NamesUnresolved,
    ControlFlowUnresolved,
    ReceiverUnresolved,
    ArgumentsUnverified,
    UnsupportedSyntax,
    UntrustedClaimsPresent,
}
impl Code {
    pub fn suppressible(self) -> bool {
        self == Self::UntrustedClaimsPresent
    }
    fn message(self) -> &'static str {
        match self {
            Self::ReviewIncomplete => "Review verification is incomplete.",
            Self::StructuralBlock => "The supplied operation is structurally blocked.",
            Self::Unsupported => "The supplied review has unsupported coverage.",
            Self::InputBudget => "The review budget was exceeded.",
            Self::MissingWhere => "The operation has no caller predicate.",
            Self::UnknownProof => "Scope proof is unknown.",
            Self::BulkWrite => "The operation has broad write scope.",
            Self::WeakPredicate => "The predicate has incomplete scope proof.",
            Self::ReadWithoutLimit => "The read has no explicit limit.",
            Self::ReadProjection => "Review the read projection.",
            Self::InsertUnverified => "Insert semantics are unverified.",
            Self::IdentityUnverified => "Source identity is unverified.",
            Self::ContextUnverified => "Source context is unverified.",
            Self::NoCandidates => "No syntax candidates were established.",
            Self::MacroUnexpanded => "Macros remain unexpanded.",
            Self::AttributeUnevaluated => "Attributes remain unevaluated.",
            Self::NamesUnresolved => "Names remain unresolved.",
            Self::ControlFlowUnresolved => "Control flow remains unresolved.",
            Self::ReceiverUnresolved => "A receiver remains unresolved.",
            Self::ArgumentsUnverified => "Arguments remain unverified.",
            Self::UnsupportedSyntax => "Syntax remains unsupported.",
            Self::UntrustedClaimsPresent => "External display claims were supplied.",
        }
    }
    fn wire(self) -> &'static str {
        match self {
            Self::ReviewIncomplete => "review_incomplete",
            Self::StructuralBlock => "structural_block",
            Self::Unsupported => "unsupported",
            Self::InputBudget => "input_budget",
            Self::MissingWhere => "missing_where",
            Self::UnknownProof => "unknown_proof",
            Self::BulkWrite => "bulk_write",
            Self::WeakPredicate => "weak_predicate",
            Self::ReadWithoutLimit => "read_without_limit",
            Self::ReadProjection => "read_projection",
            Self::InsertUnverified => "insert_unverified",
            Self::IdentityUnverified => "identity_unverified",
            Self::ContextUnverified => "context_unverified",
            Self::NoCandidates => "no_candidates",
            Self::MacroUnexpanded => "macro_unexpanded",
            Self::AttributeUnevaluated => "attribute_unevaluated",
            Self::NamesUnresolved => "names_unresolved",
            Self::ControlFlowUnresolved => "control_flow_unresolved",
            Self::ReceiverUnresolved => "receiver_unresolved",
            Self::ArgumentsUnverified => "arguments_unverified",
            Self::UnsupportedSyntax => "unsupported_syntax",
            Self::UntrustedClaimsPresent => "untrusted_claims_present",
        }
    }
}
fn decode<T: for<'a> Deserialize<'a>>(bytes: &[u8]) -> Result<T> {
    // Pre-materialization streaming check: bytes/depth/nodes/decoded keys and strings.
    crate::operation_spec::bounded::check(bytes)?;
    serde_json::from_slice(bytes).map_err(|_| Error::Invalid)
}
fn label(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 128
        || s.trim().is_empty()
        || s.contains('*')
        || s.chars().any(char::is_control)
    {
        Err(Error::Invalid)
    } else {
        Ok(())
    }
}
impl Input {
    fn review(&self) -> Result<Full> {
        label(&self.target)?;
        label(&self.context)?;
        label(&self.scope)?;
        Ok(match self.format {
            Format::Plan => Full::Review(review::plan_source(self.source.as_bytes())?),
            Format::Spec => Full::Review(review::operation_spec(self.source.as_bytes())?),
            Format::Manifest => Full::Review(review::manifest(self.source.as_bytes())?),
            Format::Rust => {
                #[cfg(feature = "source-review")]
                {
                    Full::Rust(
                        crate::source_review::review(self.source.as_bytes()).map_err(
                            |e| match e {
                                crate::source_review::Error::Invalid => Error::Invalid,
                                crate::source_review::Error::Budget => Error::Budget,
                            },
                        )?,
                    )
                }
                #[cfg(not(feature = "source-review"))]
                {
                    return Err(Error::Unsupported);
                }
            }
        })
    }
}
fn record(input: Option<Input>, full: Full) -> Record {
    let diagnostics = match &full {
        Full::Review(r) => r.ci_codes(),
        #[cfg(feature = "source-review")]
        Full::Rust(r) => r.ci_codes(),
    };
    Record {
        input,
        full,
        diagnostics,
    }
}
impl InputSet {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let w: SetWire = decode(bytes)?;
        if w.kind != "quent_review_inputs"
            || w.version != 1
            || w.analysis_version != ANALYSIS_VERSION
        {
            return Err(Error::Unsupported);
        }
        if w.inputs.len() > MAX_ENTRIES {
            return Err(Error::Budget);
        }
        let records = w
            .inputs
            .into_iter()
            .map(|i| {
                let full = i.review()?;
                Ok(record(Some(i), full))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            selection: w.selection,
            records,
        })
    }
    /// Application results retain the original review findings. Exact original
    /// facts/context are unavailable here: diff is unknown, suppression unavailable.
    pub fn inspections(inputs: &[&quent_core::Inspection<'_>]) -> Result<Self> {
        if inputs.len() > MAX_ENTRIES {
            return Err(Error::Budget);
        }
        Ok(Self {
            selection: Selection::ExplicitSet,
            records: inputs
                .iter()
                .map(|i| record(None, Full::Review(review::inspection(i))))
                .collect(),
        })
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BaselineWire {
    kind: String,
    version: u8,
    analysis_version: u8,
    entries: Vec<Suppression>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Suppression {
    code: String,
    input: Input,
    reason: String,
    owner: String,
    expires: u64,
}
/// Sensitive display preferences. Only the new claims-notice advisory is allowed;
/// ALL original findings and verification failures remain non-suppressible.
pub struct Baseline {
    entries: Vec<Suppression>,
}
impl Baseline {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let w: BaselineWire = decode(bytes)?;
        if w.kind != "quent_review_baseline"
            || w.version != 1
            || w.analysis_version != ANALYSIS_VERSION
        {
            return Err(Error::Unsupported);
        }
        if w.entries.len() > MAX_ENTRIES {
            return Err(Error::Budget);
        }
        for (n, e) in w.entries.iter().enumerate() {
            if e.code != Code::UntrustedClaimsPresent.wire()
                || e.expires == 0
                || e.expires > MAX_TIME
                || e.reason.trim().is_empty()
                || e.reason.len() > 256
                || e.owner.trim().is_empty()
                || e.owner.len() > 128
                || e.reason.chars().any(char::is_control)
                || e.owner.chars().any(char::is_control)
                || w.entries[..n]
                    .iter()
                    .any(|p| p.input == e.input && p.code == e.code)
            {
                return Err(Error::Invalid);
            }
            e.input.review()?;
        }
        Ok(Self { entries: w.entries })
    }
    pub fn empty() -> Self {
        Self { entries: vec![] }
    }
}
/// Explicit application/CLI time, Unix seconds, exclusive expiry. JSON cannot
/// supply this argument; absent/invalid current time disables every suppression.
#[derive(Debug, Clone, Copy)]
pub struct Time(u64);
impl Time {
    pub fn unix_seconds(value: u64) -> Result<Self> {
        if value > MAX_TIME {
            Err(Error::Invalid)
        } else {
            Ok(Self(value))
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Diff {
    Added,
    Unchanged,
    Removed,
    Unknown,
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Diagnostic {
    code: Code,
    diff: Diff,
    hidden: bool,
}
impl Diagnostic {
    pub fn code(&self) -> Code {
        self.code
    }
    pub fn diff(&self) -> Diff {
        self.diff
    }
    pub fn hidden(&self) -> bool {
        self.hidden
    }
}
/// Owns ALL processed head and base results and every diagnostic, including hidden.
/// No execution/gate API accepts any public projection of this object.
pub struct Report {
    head: InputSet,
    base: InputSet,
    diagnostics: Vec<Diagnostic>,
    exit: u8,
    public_budget_exceeded: bool,
}
fn multiplicity(set: &InputSet, input: &Input) -> usize {
    set.records
        .iter()
        .filter(|r| r.input.as_ref() == Some(input))
        .count()
}
fn classify(r: &Record, own: &InputSet, other: &InputSet, absent: Diff) -> Diff {
    let Some(i) = &r.input else {
        return Diff::Unknown;
    };
    if multiplicity(own, i) != 1 {
        return Diff::Unknown;
    }
    match multiplicity(other, i) {
        0 => absent,
        1 => Diff::Unchanged,
        _ => Diff::Unknown,
    }
}
/// Full head results determine exit, before diff, presentation budget or hiding.
/// No pass exists: Git coverage and runtime/identity/freshness remain unverified.
pub fn compare(
    head: InputSet,
    base: InputSet,
    baseline: &Baseline,
    now: Option<Time>,
) -> Result<Report> {
    let mut diagnostics = Vec::new();
    for r in &head.records {
        let diff = classify(r, &head, &base, Diff::Added);
        for &code in &r.diagnostics {
            let hidden = code.suppressible()
                && r.input.as_ref().is_some_and(|input| {
                    multiplicity(&head, input) == 1
                        && baseline.entries.iter().any(|e| {
                            e.input == *input
                                && now.is_some_and(|t| {
                                    e.expires > t.0 && e.expires - t.0 <= MAX_LIFETIME
                                })
                        })
                });
            diagnostics.push(Diagnostic { code, diff, hidden });
        }
    }
    for r in &base.records {
        let diff = classify(r, &base, &head, Diff::Removed);
        if diff != Diff::Unchanged {
            for &code in &r.diagnostics {
                diagnostics.push(Diagnostic {
                    code,
                    diff,
                    hidden: false,
                });
            }
        }
    }
    if diagnostics.len() > MAX_DIAGNOSTICS {
        return Err(Error::Budget);
    }
    let public_budget_exceeded = diagnostics.len() > PUBLIC_DIAGNOSTICS;
    let exit = if public_budget_exceeded {
        2
    } else if head.records.iter().any(|r| r.full.exit() == 5) {
        5
    } else if head.records.iter().any(|r| r.full.exit() == 4) {
        4
    } else {
        3
    };
    Ok(Report {
        head,
        base,
        diagnostics,
        exit,
        public_budget_exceeded,
    })
}
#[derive(Serialize)]
struct View<'a> {
    kind: &'static str,
    version: u8,
    analysis_version: u8,
    state: &'static str,
    exit: u8,
    selection: Selection,
    explicit_inputs_processed: usize,
    base_inputs_processed: usize,
    explicit_set: &'static str,
    git_head: &'static str,
    runtime: &'static str,
    authority: &'static str,
    public_budget_exceeded: bool,
    diagnostic_count: usize,
    hidden_count: usize,
    diagnostics: &'a [Diagnostic],
    head_results: Vec<&'a Full>,
    base_results: Vec<&'a Full>,
}
impl Report {
    pub fn exit_code(&self) -> u8 {
        self.exit
    }
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    fn view(&self) -> View<'_> {
        let visible = !self.public_budget_exceeded;
        View {
            kind: "quent_review_ci",
            version: 1,
            analysis_version: ANALYSIS_VERSION,
            state: if self.public_budget_exceeded {
                "budget"
            } else {
                "incomplete"
            },
            exit: self.exit,
            selection: self.head.selection,
            explicit_inputs_processed: self.head.records.len(),
            base_inputs_processed: self.base.records.len(),
            explicit_set: if self.head.records.is_empty() {
                "missing"
            } else {
                "processed"
            },
            git_head: "unverified",
            runtime: "unverified",
            authority: "not_granted",
            public_budget_exceeded: self.public_budget_exceeded,
            diagnostic_count: self.diagnostics.len(),
            hidden_count: self.diagnostics.iter().filter(|d| d.hidden).count(),
            diagnostics: if visible { &self.diagnostics } else { &[] },
            head_results: if visible {
                self.head.records.iter().map(|r| &r.full).collect()
            } else {
                vec![]
            },
            base_results: if visible {
                self.base.records.iter().map(|r| &r.full).collect()
            } else {
                vec![]
            },
        }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.view()).expect("fixed CI output")
    }
    pub fn to_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.view()).expect("fixed CI output")
    }
    /// Minimal SARIF 2.1.0 subset: fixed tool, ruleId, level and message.text only.
    /// No URI, locations, arbitrary properties, hashes or suppression reasons.
    pub fn to_sarif(&self) -> String {
        #[derive(Serialize)]
        struct Message {
            text: &'static str,
        }
        #[derive(Serialize)]
        struct Finding {
            #[serde(rename = "ruleId")]
            rule_id: &'static str,
            level: &'static str,
            message: Message,
        }
        let mut results = vec![Finding {
            rule_id: "review_incomplete",
            level: "error",
            message: Message {
                text: Code::ReviewIncomplete.message(),
            },
        }];
        if self.public_budget_exceeded {
            results.push(Finding {
                rule_id: "input_budget",
                level: "error",
                message: Message {
                    text: Code::InputBudget.message(),
                },
            });
        } else {
            results.extend(
                self.diagnostics
                    .iter()
                    .filter(|d| !d.hidden && d.diff != Diff::Removed)
                    .map(|d| Finding {
                        rule_id: d.code.wire(),
                        level: if d.code.suppressible() {
                            "note"
                        } else {
                            "error"
                        },
                        message: Message {
                            text: d.code.message(),
                        },
                    }),
            );
        }
        serde_json::json!({"version":"2.1.0","runs":[{"tool":{"driver":{"name":"Quent"}},"results":results}]}).to_string()
    }
}
impl fmt::Debug for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
redacted!(InputSet, Baseline);
