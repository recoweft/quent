Current PR3 additions: [finite owned atomic results, version/RETURNING IR and borrowed Tx boundaries](atomic-results-v1.md). Earlier PR-specific absence statements below describe their historical baseline.

# QT-AI-02 PR1: immutable application snapshots

Current QT-AI-02 PR2 policy preparation, finite support/refusal matrix and evidence:
[policy preparation v1](policy-preparation-v1.md). Implementation-status paragraphs describe their named historical PR; the
integrated requirements remain current. Raw Plan/snapshot APIs are non-enforcing
and non-executable.

`quent-core` now supplies owned private-field `Settings`, `PolicySet`,
`DatabaseBinding`, `SchemaSnapshot`, `TenantSnapshot`, `ApplicationContext`,
`DatabaseContext` and DB-free `PreparationContext`. There is no mutable global,
registry, ambient tenant, clock, random identity, callback, Any or metadata map.
Clone deep-copies immutable configuration; it does not clone an execution handle.
No executor/validated handle/transaction is introduced. The production Plan and
inspect APIs still use their existing private finalized record, unchanged.

## Binding and application boundary

A binding retains the exact application database identity, nonsecret routing
address/key, catalog identifier and dialect. Identities are nominal Rust types,
owned 1..128-byte ASCII alphanumeric or `_ - . : /` tokens (without spaces).
Catalog and column/table identifiers use the existing Ident grammar. Do not put
credentials or DSNs in routing keys. These are configured application assertions:
even exact equality does not authenticate a physical database, server/session,
current routing destination, caller or authorization. No digest replaces contents.

Settings/policy/schema each retain their own identity and nonzero u64 generation.
Schema additionally owns up to 16 ScopeFacts snapshots, including their complete
finite table/column/key/context assertions. Duplicate exact table/alias identities
or dialect disagreement are refused. ScopeFacts remains ApplicationAsserted and
unsupported table semantics stay unsupported; no LiveVerified variant is added.
An empty fact list is an unconfirmed schema declaration, not schema coverage.
The carrier does not supply missing facts to Plan::inspect automatically.

ApplicationContext::application_supplied is for authenticated/authorized server
code. Its constructor performs structural validation only. The app must never
forward HTTP/AI claims to these constructors as if construction authenticated them.
There is no serde, FromStr, public-summary restoration, freshness/reason/allow input
or permission output. Subject is an application identity, not a verified session.
`Option` distinguishes missing schema/subject/tenant from supplied values; empty
identities and generation zero are refused. `presence()` reports only three presence
bits, never readiness, trusted authentication, allow or current/live status.

Tenant retains its exact target, column (table/path/type/nullability) and native
value. Only nonnullable signed SQL integer columns/values with exact matching types
are supported: PostgreSQL I16/I32/I64, MySQL I8/I16/I32/I64. NULL, nullable,
unsigned, text, bool, coercion and aliases are unsupported. Zero/negative values are
ordinary explicit signed values. This is not encoding or live schema verification.

All composed target tuples must match exactly. If supplied, tenant column must
match every tenant declaration in the settings. Thus one tenant-bearing context
currently supports one exact tenant column; a multi-table policy set with distinct
tenant columns cannot be combined with one TenantSnapshot (fixed Policy error).
Missing tenant stays missing. Where schema facts for a declared table are supplied,
policy/tenant columns must occur with exactly matching type/nullability. Missing
table facts stay unconfirmed; presence is not evidence of coverage. Matching names,
identity strings and generations alone cannot override different retained contents.

## Settings, declarations and derivation

Strict is the only mode and the initial default. There is no compatibility/allow
mode. PolicySet owns at most 256 declarations in input order:

| Declaration | Accepted foundation | Application in this PR |
| --- | --- | --- |
| Tenant | supported nonnullable signed column | Unimplemented |
| SoftDelete | nonnullable Bool column | Unimplemented |
| Pii | supported planner column type/nullability | Unimplemented |
| RequiredFilter | supported planner column type/nullability | Unimplemented |

Aliases and PostgreSQL unsupported native integer types are refused. One declaration
per table-path/column name is accepted; duplicates, conflicting types and multiple
tenant columns for one table are refused. This conservative foundation also refuses
combining two different declaration kinds for the same column. Empty declarations
mean absence, never allow. No policy verdict, SQL rewriting, column enforcement,
batch/upsert handling or tenant auto-binding is implemented. QT-AI-02 PR2 owns those.

`Settings::next` requires strictly increasing settings AND policy generations,
preserving settings/policy identities and target; wraparound is not provided.
`DatabaseContext::with_settings` enforces the same update boundary. These methods
return new snapshots. New root constructors accept explicit application generations;
there is no global registry that can enforce a globally monotonic/current history.
`with_application` explicitly replaces only schema/subject/tenant at the same target,
validating the new composition. It does not mutate the original or its siblings.
It may deliberately supply an old schema assertion; no freshness guarantee follows.
Different settings identities or targets require a new root context.

`prepare()` on a DatabaseContext deep-copies its exact snapshot; recursive preparation
also preserves it. PreparationContext has no change/execute/begin/commit/rollback API.
Its borrowed snapshot can be used to explicitly construct another DatabaseContext.
`same_snapshot` compares all retained content, including types, values, schema facts,
declarations and generations, not a digest or generation alone. It is a DB-free
comparison only, not QT-C12 dispatch revalidation or an operation/permit binding.
QT-AI-03+ must connect the actual executor/Tx and recheck current exact context,
operation, provider decision and other required bindings immediately before dispatch.

## Output and bounded failures

Every new sensitive carrier has constant redacted Debug and no Display/serde.
Fixed ContextError classifications Identity/Generation/Binding/Policy/Type/
Unsupported/Budget retain no input, arbitrary free text or source error. Errors
implement fixed Display and std::error::Error with no source. Identities have no
ordinary value getters. Explicit local `DatabaseContext::sensitive_source()` follows
the existing Plan SensitiveSource boundary: borrowed exact target, versions,
declarations, schema facts, subject and tenant are sensitive. Never put getter
results into CLI/MCP/logs/public summaries; no exporter is supplied. Public presence
bits and Strict mode do not reconstruct a context or authorize execution.
Owned equality/Clone preserve snapshot content and expose no mutable interior API.

## Evidence and remaining acceptance

Five new Core tests use production APIs and literal independent expectations for
both dialects: QT-C11 A/B settings/policy/tenant/subject isolation, input clone changes,
sibling/nested inheritance and old/new generation snapshots; QT-C12 target/address/
catalog/dialect/subject/value/generation/content mismatches and missing context;
unsupported types, duplicate policies, identity/collection budgets and schema column
inconsistency; retained exact schema facts with ApplicationAsserted inspection;
QT-C15 redacted carriers/errors and explicit sensitive inspection. Reusing identical
identity/generation labels with different declaration/schema content does not compare
equal. The source fixture and expected dispatch/Tx results are not copied as results.

One new facade integration test reads QT-C08's documentary allow claim but supplies
no server tenant/subject/schema: both dialect snapshots remain missing. Six new
compile-fail doctests check private settings, JSON-like FromStr input, Debug
restoration, preparation commit, actual serde Deserialize and Serialize boundaries.
The serde checks use existing facade dev dependencies, not missing imports.
This is limited input/output boundary evidence, not full JSON/manifest/provider or
runtime gate acceptance. All 24 fixture runtime_pending markers remain unchanged.
No recorder dispatch/refusal/rollback events are fabricated or counted as guarantees.

Real PostgreSQL/MySQL tests are nonapplicable to this pure carrier change: no SQL,
driver, encoding, connection, provider, transaction or database side effect is added.
Server versions/settings, live schema, physical identity, row counts, Tx outcomes,
Windows local execution and public JSON/SARIF/MCP serializers remain unverified.
No dependencies/features/lockfile/toolchain/MSRV/edition/publish=false change is needed.

## Local verification

Ubuntu 24.04.5 / WSL2, rustc/cargo 1.94.0: fmt, workspace build/test (default
and all features), all-target/all-feature clippy with warnings denied, all-feature
doc with warnings denied, both adapter + runtime-tokio + tls-rustls compatibility
checks and all 16 subsets of the four facade features passed with --locked --offline.
52 tests passed per workspace test run (40 regressions + 5 Core carrier tests +
1 source-claim integration + 6 compile-fail additions); zero failed/ignored in final
runs. Core dependency tree has no dependencies; default normal workspace tree has
only local crates. Hosted Ubuntu/Windows CI is separate evidence read from the PR;
local checks do not claim Windows execution or real DB/Tx acceptance.

## QT-AI-03 PR1 runtime boundary

[Immediate SQLx dispatcher](sqlx-dispatch-v1.md) adds the current shared Strict gate,
owned-session context binding, finite native adapters and bounded redacted results.
Its support/refusal and evidence tables supersede historical statements above about
the absence of execution. Raw Plan/source/preparation remain non-executable inputs.
INSERT remains InsertUnverified/High and is refused without a provider. All full
runtime_pending markers remain: limited live PR1 evidence is recorded separately
from PR2 handles, PR3 Tx/atomicity and QT-AI-09 authorization responsibilities.

## QT-AI-03 PR2 boundary

[Consumed validation](owned-validation-v1.md) records the current exact-operation/
context/session/deadline checks, ownership and async stage contract. This adds finite
QT-C12/C13/C14 evidence through the same production dispatcher without changing
Core planning, PR1 refusal/native gates or any full runtime_pending marker.

## QT-AI-04 PR1 output boundary

[Public views v1](public-views-v1.md) adds output-only finite projections of the
current retained record/inspection/preparation and fixed errors/result states.
Historical statements above deferring public views describe their named baseline.
No public JSON or copied display classification is execution authority. Sensitive
getters and the complete private gate remain unchanged; CLI/tracing/export remain
with PR2/owning issues. All 24 full runtime_pending markers remain.
