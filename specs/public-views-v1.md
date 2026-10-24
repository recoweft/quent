Current PR2 integration: [output boundaries v1](output-boundaries-v1.md). PR1-specific
absence statements below describe that historical baseline; the v1 register/schema is unchanged.

# QT-AI-04 planned PR1: public output v1

`Plan::public_view()`, `Inspection::public_view()` and
`PolicyPreparation::public_view()` produce owned output-only `PlanView` copies.
The raw Plan starts unevaluated/unknown, with no evidence or findings. An Inspection
projects its own retained finalized record, never a caller-supplied second Plan.
Preparation inspects its retained exact facts and final record. No metadata, SQL,
argument, context or error object is cloned into a view and later redacted.
Planning/inspection/projection/JSON are deterministic and DB/provider-free.
Core has no dependencies; no new production/dev dependency, feature, lockfile,
Rust 1.94.0/edition 2024/publish=false or exact SQLx/Tokio/futures-util pin changes.
The facade adds execution mappings only within its existing driver/runtime boundary.

## Output schema and register

[Closed JSON schema](public-views-v1.schema.json) enumerates every property/value.
Each document has numeric version `1` and a fixed kind: `quent_plan_view`,
`quent_diagnostic_view` or `quent_result_view`. `to_json()` emits compact stable
field order; Display emits the same JSON. Debug shows only the same finite fields
(Rust formatting is not the JSON wire contract). No Serialize or Deserialize trait,
FromStr/input parser, restoration/execute API, conversion into IR, context,
preparation, validated/atomic handle or permit is supplied. JSON/Debug/risk/allow/
fresh claims must never be accepted as execution-gate inputs. Future external
input parsers are a separate responsibility.

Plan fields are exactly kind/version, precision, risk, scope, scope_upper_bound,
completeness, assumptions, evidence, findings, family, projection,
projection_count, write_count, order_count, versioned, returning_count,
caller_missing_where, has_limit. Counts are structural, 0..256, including the
version assignment and RETURNING projection; COUNT is one projection, never the
COUNT result value. LIMIT contributes only a presence bit, never its bound value.
No predicate literal, bound argument count/type/value, schema generation, tenant,
subject, table/column/schema/model/alias name, path/snippet, message/reason/owner,
source error/cause/opaque metadata, SQL/value-derived fingerprint is present.

Precision is unevaluated for raw Plan, unknown for Unknown/NotApplicable scope,
and conditional_conservative for other inspected scopes. It never means live,
exact cardinality or complete SQL semantics. Completeness is the original
complete/partial/unknown finite abstraction coverage. Complete/Low/Zero/Single
never grants business authorization, existence, exactly-once, affected-row,
transaction or live-schema guarantees. Scope upper bound is 0 for Zero, 1 for
Single, the finite u16 for FiniteBound, null otherwise. Production inspection
currently bounds finite tuples at 256; caller-modified display copies may hold
any u16 and carry no proof. Assumptions are none/application_asserted; there is
no live-verified state. Risk includes unevaluated/low/medium/high/destructive/blocked.
Family is select/count/insert/update/delete; projection is columns/count/none.

Evidence register (nullable only for raw Plan):
missing_facts, inconsistent_facts, unsupported_context, unsupported_key, analysis_budget, empty_truth_set, complete_key_tuples, unbounded_predicate, unsupported_predicate, no_predicate, insert_shape.

Finding register (complete fixed-order output, at most 7, no truncation):
missing_where, unknown_proof, bulk_write, weak_predicate, read_without_limit, read_projection, insert_unverified.

MissingWhere retains precedence, including caller_missing_where before automatic
tenant guards. Missing/inconsistent facts, unsupported context/key/predicate,
partial analysis and exhausted budget retain the existing scope/completeness/risk;
no view projection weakens the internal gate. Findings and structural fields have
no public list/string setter. There is no diagnostics collection or caller-supplied
text extension. A diagnostic is one fixed code and one optional finite state.
Future display truncation must preserve unknown/block/incompleteness and be
independent of the complete internal diagnostics and execution gate.

Diagnostic code mapping is exhaustive over the following existing enums:

| Source | v1 code register |
| --- | --- |
| `PlanError` | `plan_identifier`, `plan_shape`, `plan_type`, `plan_missing`, `plan_unsupported`, `plan_budget` |
| `ContextError` | `context_identity`, `context_generation`, `context_binding`, `context_policy`, `context_type`, `context_unsupported`, `context_budget` |
| `PolicyError` | `policy_missing_schema`, `policy_identity`, `policy_coverage`, `policy_unsupported`, `policy_missing_tenant`, `policy_tenant_mismatch`, `policy_tenant_predicate`, `policy_tenant_write`, `policy_pii_access`, `policy_required_filter`, `policy_protected_write`, `policy_budget` |
| `ExecutionError` | `execution_context_mismatch`, `execution_operation_mismatch`, `execution_invalid_ttl`, `execution_expired`, `execution_missing_subject`, `execution_risk_refused`, `execution_incomplete`, `execution_unsupported`, `execution_encode`, `execution_connect`, `execution_read_failed`, `execution_decode`, `execution_result_limit`, `execution_outcome_unknown`, `execution_session_unavailable`, `execution_expectation_mismatch`, `execution_conflict` |

PolicyError::InvalidPlan delegates to the PlanError code; ExecutionError::Policy
delegates to the PolicyError mapping. Wrappers carry no free text/source.
AtomicFailure keeps that code and maps disposition to not_started/rolled_back/
outcome_unknown. DiagnosticView implements Error with no source; arbitrary
std::error::Error/Display/source chains cannot be ingested. ResultView contains
only observed/uncommitted/committed from ExecutionResult, omitting rows, COUNT and
affected counts entirely. Display copies may be edited using only finite enums;
no edit changes the original operation, inspection, context, Strict gate, consumed
handle or result. State alone cannot authorize an atomic operation.

## Evidence and boundaries

`quent/tests/public_views.rs` uses both production dialects, independent complete
JSON expectations/key allowlists, repeated serialization, typed/NULL/text and
SQL/name/alias/subject/tenant/path canaries. Raw, prepared, conditional, partial,
budget, unknown, MissingWhere/Blocked, High/Destructive, version and RETURNING
boundaries are checked. JSON is independently parsed by the existing test-only
serde_json; fixed ASCII strings need no input escaping. No serializer uses source
Debug or arbitrary strings. Every numeric field is a bounded integer; JSON has no
NaN/floats. The schema is documentary output-only, not an authority parser.

`owned_api.rs` retains 29 negative programs and 2 controls, adds a public view
control, 25 output-to-authority conversion refusals, 3 execution-boundary refusals
and 3 deserialize/arbitrary-error refusals, checking E0277/E0308 without missing
imports/types. `execution/tests.rs` independently checks result and atomic failure
JSON states, existing errors/limits and live shared-gate noninterference: views
produce no send events, forged Low cannot unblock original missing-WHERE, forged
Blocked cannot change the actual accepted atomic operation. Existing live tests
still observe production dispatch/commit/rollback and independent database state;
the support recorder is not used as a dispatcher guarantee.

QT-C15 is limited here to implemented production view JSON/Debug/Display/Error
boundaries and restoration refusal. QT-C01 deterministic DB-free structure,
C08 output nonpromotion, C12/C13/C14 carrier/execution refusals and regression,
and C16 version/RETURNING/atomic state preservation are supporting evidence.
Both digest-pinned disposable DB regression suites are separate from the DB-free
canary evidence. No new DB semantics or side effect was added. Full 24-case
runtime_pending remains unchanged, including provider/command and real network
completion-fault responsibilities. Existing synthetic fault injection is not a
measurement of a real lost commit/rollback reply.

PR2 and owning issues retain CLI/tracing/SARIF/MCP all-path output integration,
local detailed export, new-path/no-overwrite/Windows ACL and their canary tests.
Sensitive SQL/values remain behind existing explicit getters, without new public
Serialize/Display. CLI remains fixed non-success QUENT_NOT_IMPLEMENTED with empty
stdout. No unfinished exporter or output integration is claimed complete. Existing
nonoptional CI jobs are unchanged; required branch-protection enforcement is not
claimed. Actual local and hosted verification is recorded in the PR/report.

## Local verification (2026-10-24, Ubuntu 24.04.5 / WSL2)

Pinned rustc 1.94.0: fmt, default/all-feature workspace build, default tests
(78 passed), all-feature DB-free selection (96 passed), warning-denied all-target
clippy and all-feature rustdoc passed with --locked --offline. The existing
`c12_exact_target_and_context_missing_never_become_live_or_permission` test matches
the broad `--skip live_` substring and is covered by default tests and a separate
all-feature Core context run (5 passed). Two dedicated live suites passed separately;
thus all 91 baseline tests plus 8 new test functions are covered (99 unique tests,
not 96+5+2 distinct tests). The diagnostic-checked harness runs 60 negative programs
(29 original + 31 added) and 3 successful controls. Both PostgreSQL 16.14 and MySQL
8.4.8 disposable regressions verified cleanup of their containers/volumes. Live
view assertions cover commit, borrowed uncommitted, actual expectation-mismatch
rollback and synthetic OutcomeUnknown; a real lost-completion-reply guarantee is
still unverified. All 16 feature subsets and Core/default normal dependency trees
passed; Core has no dependencies and default normal tree has only local crates.
CLI was run separately: exit 1, empty stdout, fixed QUENT_NOT_IMPLEMENTED stderr.
Hosted Ubuntu/Windows and live CI results must be read from the actual PR head;
these local measurements do not establish branch protection or local Windows ACLs.
