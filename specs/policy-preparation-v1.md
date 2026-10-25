Current PR3 additions: [finite owned atomic results, version/RETURNING IR and borrowed Tx boundaries](atomic-results-v1.md). Earlier PR-specific absence statements below describe their historical baseline.

# QT-AI-02 PR2: finite DB-free policy preparation

`PreparationContext::prepare_operation(&Operation)` is the single application-only
entry for SELECT, COUNT, single INSERT, UPDATE and physical DELETE. It returns an
immutable `PolicyPreparation`, not a validated operation, permit, transaction or
execution handle. Raw `Operation::finalize` remains an unchecked planning API;
there is no conversion of raw Plan or public output into policy preparation.
All dynamic Table/Column construction uses the same checks. There is no separate
renderer, SQL parser, public JSON reader, dispatcher or provider callback.

The result privately retains the original operation, the exact immutable database
context, the selected schema facts and the final Plan. Guards modify a bounded copy
of the IR before the sole existing renderer finalizes it. `inspect()` borrows that
same final record and its retained facts, including final tree, SQL, ordered native
args and metadata. It takes no caller replacement facts. SensitivePolicySource is
an explicit local sensitive boundary for original operation/context/final source;
never export those getters to logs/CLI/MCP/public summaries. No serde/restoration,
interior mutation or arbitrary policy callback/Any/reason/allow input is added.

## Identity and conditional assumptions

Preparation requires a schema snapshot with exact table coverage, exact native
column type/nullability for every used and applicable policy column, and an explicit
`OrdinaryBaseTableWithoutEffects` application assertion. Missing coverage is refusal,
not absence of policy. Constructor success, identity/generation strings and
same_snapshot do not authenticate the user, physical target, live schema or current
settings. The retained complete facts and target tuple are application assertions.
The app must supply authenticated/authorized tenant context and correct database
semantics. No live verification/freshness or final dispatch revalidation is promised.

The supported identity subset is deliberately narrower than raw plans:

| Identity | Preparation behavior |
| --- | --- |
| Explicit two-component schema/table (PG) or database/table (MySQL), no alias | Exact retained identity required |
| MySQL first component differing from binding catalog | Identity refusal |
| Unqualified table/search_path resolution, aliases, catalog.schema.table | Unsupported; no implicit resolution |
| Identifiers outside nonempty ASCII `[a-z0-9_]+` | Unsupported in preparation; raw Plan retains its escaped identifier capabilities |
| Any noncanonical table/column policy declaration | Unsupported even if it would otherwise appear inapplicable |
| Same table basename policy under another path | Identity refusal; no fallback to policy-free operation |
| Missing schema/table/column, inconsistent type/nullability | MissingSchema/Coverage refusal |
| Unsupported table context (views, triggers, RLS, unknown effects) | Unsupported |

The canonical spelling restriction prevents MySQL case folding from making a second
nominal column an alias for the tenant column. Exact two-part paths avoid implicit
search_path/current database resolution. Two namespaces with the same table basename
and different declarations are conservatively refused. PostgreSQL catalog remains
part of the exact configured context (not a SQL path component); physical database
identity is still the application's responsibility. PR1's one exact signed tenant
column per tenant-bearing context, no alias and exact target composition rules remain.

## Finite enforcement table

Policies are explicit finite declarations, one kind per exact column as in PR1.
Empty declaration sets are absence, never authorization. Readonly and Generated
are the only declaration additions; both protect supplied INSERT/UPDATE columns.
They do not implement generated SQL expressions, defaults or live generated semantics.

| Policy | SELECT / COUNT | Single INSERT | UPDATE | DELETE |
| --- | --- | --- | --- | --- |
| Tenant | Validate all explicit occurrences, append equality guard | Fill absent column only; exact explicit value required | Validate predicate, append guard; reject assignment even if equal | Validate predicate, append guard |
| SoftDelete nonnullable Bool | Append `column = false` outside caller | No filter/default synthesis | Append outside caller | Append outside caller; remains physical DELETE |
| Pii | Refuse projection/filter/order access; COUNT without column access permitted | Refuse supplied PII | Refuse supplied/predicate PII | Refuse when table has PII, as whole row is affected |
| RequiredFilter | Require caller proof | Refuse: no caller WHERE proof | Require caller proof | Require caller proof |
| Readonly / Generated | No additional restriction | Refuse explicit supplied column | Refuse assignment | No additional restriction |

Tenant's initial domain is the PR1 exact nonnullable signed native SQL integer
subset: PG I16/I32/I64, MySQL I8/I16/I32/I64. No coercion, NULL, unsigned, text,
nullable or unknown tenant types. Missing trusted tenant refuses tenant-bearing
operations. Every explicit occurrence is checked, including contradictory/dead
branches: equality must equal the trusted native value; IN checks every element
(empty IN remains false). Range/Ne/NOT/IS NULL on tenant refuse rather than becoming
proof through an added guard. NULL/wrong native value refuses. There is no manual
tenant-proof mode. Missing tenant in an OR arm is safely filled by the outer guard;
no branch is rewritten or discarded and no explicit mismatch is overwritten.

Final predicate is `AND(entire caller, guards in declaration order)`, or
`AND(guards)` without a caller. SQL parentheses retain the whole caller tree. Thus
`(tenant=T1 AND key=1) OR key=2` becomes caller AND tenant=T1 on every possible branch.
Native args remain in renderer occurrence order: UPDATE data first, then complete
caller, guards, and read limit last. INSERT appends the missing tenant pair after
all caller pairs. All duplicate/native type/missing/shape checks still run in the
sole renderer. SQL is never modified after finalize or inspection.

RequiredFilter uses a conservative compositional proof: supported signed nonnullable
same-native-type non-NULL equality establishes the required column; AND needs at
least one child establishing it; OR needs every child establishing it. NOT, range,
NULL, IN, nullable/text/bool and mere column occurrence do not establish proof.
False/contradictory arms do not waive it. Auto tenant/soft-delete guards cannot supply
caller proof. A separate equality outside an OR can cover the whole OR. This does
not authorize PII or business access; PII reason/allow/provider integration is absent.

Batch/upsert/RETURNING/join/Raw/CTE/subquery have no production IR path and remain
unsupported. No first row or first candidate is used to claim batch/upsert checks.

## Structural risk, budgets and output

The final record additionally retains the original missing-WHERE write bit.
Inspection reports Blocked/MissingWhere (non-suppressible) for original WHERE-less
UPDATE/DELETE even after automatic guards. Final scope still describes the final
tree, so bounded final scope never removes the original structural block.
Tenant-wide/partial composite remains broad without a complete key. Unsupported or
partial analysis remains unknown/partial/high; bool soft-delete currently makes
surviving scope analysis partial under the existing finite scope rules. Policy
preparation success, Single/Zero/Low and copied display values grant no permission.
No suppression or mutable public classifications can weaken the private result.

Before any recursive clone, raw finalize and preparation share a bounded borrowed
prewalk: depth 16 edges, 1024 nodes, 256 children/IN elements, 256 projection/write/
order entries, 1024 args, 4096 text bytes. It refuses overlarge input before cloning
operation metadata/projection. Preparation also refuses exhausted TRUE-candidate
analysis (256 branches / 65,536 conjunction pairs), before and after guards. Added
filters/tenant pairs must fit the original finalize budgets, including SQL 131072
bytes; no truncation or partial acceptance. Caller construction/allocation/drop of
an arbitrary Draft is outside these preparation budgets.

All new carriers have redacted Debug. PolicyError exposes only finite input-free
codes (including finite PlanError), fixed Display and no source chain. No SQL, names,
tenant, subject, values or arbitrary error are retained in errors. Sensitive data
requires explicit SensitiveSource/SensitivePolicySource access.

## Evidence and remaining responsibility

Eleven Core tests exercise both dialects with independent literal SQL/ordered args,
final tree/metadata/family and context/original immutability expectations. Coverage
includes QT-C02–06/C07/C08/C10/C11/C12/C15: outer-AND leak prevention, all explicit
OR branches and IN candidates, late mismatches, same-value tenant updates, missing/
matching/mismatching/NULL INSERT values, protected writes, schema/path/alias/native
type/canonical identity refusal, missing tenant, soft-delete, PII and RequiredFilter
positive/negative rules, A/B isolation, retained block, depth/nodes/args/columns and
branch budgets, redaction and exact sensitive correspondence. A separate finite
oracle enumerates 64 caller shapes × 9 tenant/key rows × both dialects and checks
final truth equals original truth AND trusted tenant. This is finite regression
evidence, not live SQL semantics or a general theorem prover.

The common fixture bridge connects 15 source shapes × both dialects from
QT-C02–07/C10 to the real preparation API with explicit separate application
assumptions and independent expected structural results. Both QT-C06 shapes use
this API's automatic mode; the fixture's future manual-mode rejection is not claimed.
QT-C07 lossless-values (decimal/missing) and QT-C09 batch are deliberately excluded;
existing native value refusals and single INSERT tests do not prove batch handling.
QT-C08's documentary allow/fresh claims additionally cannot satisfy preparation's
schema requirement. QT-C09 full batch acceptance, QT-C11 Tx inheritance and QT-C12
final dispatch mismatch remain pending. All 24 full runtime_pending markers remain.
Four new compile-fail examples check private mutation/execution and actual serde
Serialize/Deserialize refusal of the new carrier (existing dev dependencies only).

Real DB tests are nonapplicable to this DB-free change: no provider, driver encoding,
SQLx dispatch, connection, Tx or database side effects are added. No server versions,
settings, RLS/triggers, affected rows, rollback or commit outcomes were measured.
An empty/absent recorder is not dispatch refusal or rollback evidence. QT-AI-03/09+
must implement execution/authorization/Tx and exact current-context/operation/target/
provider comparison immediately before dispatch. Public views, OperationSpec and MCP
remain with their owning issues. Runtime acceptance cannot be cleared by these tests.

Rust 1.94.0/edition 2024, Core no dependencies, default driver/runtime independence,
exact dependency pins and publish=false are unchanged. Local verification commands
and actual hosted CI observations are recorded in the PR and RelayWeft report.

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


QT-AI-09 planned PR1: [explicit scoped authorization acquisition](authorization-contract-v1.md) adds
immutable per-DB trusted registration and consumed exact request-bound candidates.
Successful acquisition has no dispatcher/Tx/permit conversion; existing Strict
High/Destructive refusal and all 24 full runtime_pending markers remain.
