# QT-AI-11 PR3: explicit finite migration application

This application-only API is available at `manifest::live::apply` with `manifest`,
`runtime-tokio` and an explicitly selected PostgreSQL/MySQL feature. It accepts
only the original immutable PR1 `MigrationPlan` through a private typed request.
The finalized SQL, before/after schemas, authority, ordered differences, complete
phase records, conditions and step range remain the sole record. There is no
second planner, SQL input, closure executor, driver callback, borrowed transaction,
public JSON restoration, MCP/AI write/permit API, ambient DSN, CLI apply or automatic
migration. Existing UPDATE/DELETE authorization never authorizes DDL.

The delivered consultation for question
`d8b53a8a-cc48-4cdd-9c03-929f96c6e6da` specifies the following conditional trust
boundary. It does not promote PR2 evidence to complete readiness.

## Explicit application responsibilities and API

1. The application authenticates the person, their explicit intent, and the exact
   operation they approved. Construct the original typed plan and a finite Expand
   `PreflightRequest`. Constructors and Rust type success do not authenticate people.
2. Register a separate `ApplyContext` with the exact `DatabaseContext` (including
   subject), immutable adapter allocation, bounded identity/configuration (128/4096
   bytes) and nonzero configuration generation. Registration is per database.
   `with_database` explicitly derives context with the same immutable registration
   and exact target; subject/context changes still invalidate old handles.
3. Explicitly connect `PostgresApply` or `MysqlApply` using checked options and
   context. The session privately owns a fresh connection and allocation identity.
   Its `identity()` is a non-authoritative token for current snapshots, not a permit.
   No existing reader or observation can be supplied or converted to an apply session.
4. Create and retain one non-Clone/non-serde `ApplyRequest` with a bounded text ID.
   Recreating the same text creates a distinct private operation allocation and
   cannot restore that operation. Supply `Current::new`: current context/registration,
   session identity, a reference to this retained operation, exact private preflight request, five independent assertions
   in the fixed `Requirement` order, and a separate `HumanInstruction`. Each has a
   finite state, nonzero generation and process-monotonic expiry. No free-form reason
   or single allow boolean replaces this record. JSON/view/manifest Fresh does not
   construct any evidence, decision, candidate or handle.
5. `session.candidate(current, ttl)` is pure: DB/provider zero, ttl in `(0,300s]`.
   It binds the exact session/target/context/full plan/phase/range/scope/role/visibility/
   exclusion/request/conditions and registration before observation or provider I/O.
   Its deadline is no later than any initial assertion or human-instruction expiry.
6. Explicitly `prepare(candidate, current_source, budget).await`. It performs a new
   finite observation on this session's actual private connection, using the closed
   PR2 catalog reader and its shared budget/quiet driver/read-only BEGIN or START
   plus ACK ROLLBACK. Only after observation Tx cleanup succeeds does it invoke the
   separate trusted migration adapter with sensitive in-process plan/context/request/
   registration/scope/session/phase data. No executor/connection/Tx is passed.
7. The adapter checks all five residual requirements and the independent human
   instruction. `ApprovalRequest::respond` consumes the request wrapper and binds
   the decision to one private invocation allocation. Each response assertion and
   human instruction must match the independently current exact record. A stored
   decision from another invocation cannot satisfy this check. Missing adapter,
   missing subject, Unknown/Denied/mismatched/expired facts refuse before DDL.
8. `apply(handle, current_source)` consumes the non-Clone/non-Copy/non-serde handle
   when constructing the future. Failure never returns it. An unpolled future does
   no I/O. A polled application quarantines the session at its execution boundary;
   errors, timeout, cancellation and drop cannot authorize reuse or reconnect.

`CurrentSource` is a synchronous trusted application contract returning independent
bounded current snapshots. It is neither a permission issuer nor an executor.
Returning the retained initial snapshot while external state changed violates the
contract. Quent cannot detect such replay or unseen physical/store changes.
The source is consulted before and after observation/provider work, each DDL send,
each DDL/Tx await and final completion. A private polling guard also checks
independent current facts before SQLx resumes after internal preparation/protocol
waiting; revoked current state prevents resuming that driver future. Full bounded record comparison precedes the
actual `Instant::now() >= expiry` check; no application await or SQL-replacement
callback lies between the final send check and the closed driver call. Exact plan
comparison includes all retained content, not just names, SQL or hashes. All five
assertions and human state are separately checked on every boundary. A changed
current snapshot stops further DDL/COMMIT, but does not undo a MySQL ACK prefix.

The application must keep exclusion/guarantees valid while work is processing on
the server, including after client timeout/drop without completion ACK. It owns
reconciliation, confirming connection/server termination, deciding when exclusion
can safely end and authenticating any new human recovery decision. No automatic
retry, new request-ID substitution, automatic continuation or generic recovery
executor exists. External adapter authentication/transport/logging/panic hygiene
and factual assertion provenance are application responsibilities. Fixture adapters
are contract controls, not an implemented identity provider or exclusion service.

## Coverage and trust sources

| Required domain | Source | Refusal/limit |
| --- | --- | --- |
| Before columns/order/native/null/default and finite local constraints/table flags | New finite SELECT observation on the actual apply connection | Drift/Unknown/Unsupported/visibility or budget failure refuses; PR2 normalized missing default remains Unknown |
| Authenticated physical routing, declared target/role correspondence | ApplicationAttested, Requirement 0 | Target labels and CURRENT_USER alone do not authenticate physical identity |
| Complete compatible table effects, extensions and related incoming references | ApplicationAttested, Requirement 1 | Local constraint count zero alone never proves full-effects absence |
| Provenance of exact before/after default and other source declarations | ApplicationAttested, Requirement 2 | Catalog normalization never fabricates original NULL/default syntax; observed Unknown cannot be overridden |
| Necessary DDL privileges and valid visibility scope | ApplicationAttested, Requirement 3 | Real server refusal is retained; no GRANT, role switch or alternate-route retry |
| External DDL exclusion from observation start through actual DDL/Tx completion | ApplicationAttested, Requirement 4 plus exact exclusion generation | Same connection does not eliminate observation/apply gap; Quent implements no external lock manager |
| Explicit human instruction for this exact request | Independent request-bound adapter decision plus current HumanInstruction | Fact attestation never automatically authorizes a human operation |

PR2 `PreflightObservation` and view remain ConditionsObserved / overall Partial /
full_effects Unverified. They have no apply conversion. The new private observation
also remains partial; application-attested domains are not LiveObserved or
FullVerified. This is a conditional finite application gate, not a claim that the
library observed the entire schema/effects or authenticated a person.

Only constraint-free nullable signed-integer tail append is supported, following
PR1's exact dialect types and SQL. Expand works; Backfill/Contract remain NoWork or
Unsupported. Drop/type/null/default changes, backfill writes, destructive contract,
unknown/unsupported effects/engine, constrained tables and any unsupported component
refuse the entire request before DDL. An adapter cannot waive structural refusal,
unknown observation, native/capability or resource bounds. Destructive migrations
would require a separate permission boundary and are absent here.

## Completion, progress and recovery limits

| Boundary | PostgreSQL 16.14 | MySQL 8.4.8 |
| --- | --- | --- |
| Observation | Fixed read-only repeatable-read BEGIN, finite SELECT, ACK ROLLBACK | Fixed read-only START, finite SELECT, ACK ROLLBACK; no whole-catalog MVCC claim |
| Apply begin | Separate explicitly owned BEGIN; failed/unknown response = BeginUnknown | No business Tx wrapper around ALTER |
| DDL | Exact retained final SQL in order and end-of-result ACK; DDL row counts have no CRUD exact cardinality contract | Same; ALTER implicit commit means ACK prefix can persist |
| All steps ACK + valid current | COMMIT ACK required for Committed | ImplicitCommitPrefix includes all steps if acknowledged==steps and unsent==0 |
| Server error | rejected_step retained; explicit ROLLBACK ACK required for RolledBack | StepRejected, rejected step distinct from unacknowledged step; earlier ACKs may persist |
| Unacknowledged step | StepUnknown; cleanup may yield RolledBack or RollbackUnknown | StepUnknown, step may have applied; ACK prefix/unsent suffix remain separate |
| COMMIT response unknown | CommitUnknown, never auto retry/rollback-to-success | Not applicable |
| Final current check fails after COMMIT ACK | CommitAcknowledgedCurrentInvalid, refusal retained; ACK is not erased | Prefix remains, refusal retained |

`ApplyOutcome` and `session.progress()` expose fixed completion/error enums and
bounded total/attempt/ACK/rejected/unknown/unsent indices/counts only. Their output-only
[JSON schema](migration-apply-v1.schema.json) identifies trust **sources**, not
unconditional fact/authorization success. `evidence_record` distinguishes no
acquisition from a previously acquired finite observation and request-bound
application decision; current validity still requires the separate completion and
refusal fields. No result view can restore authority. An attempt is
a SQLx call boundary, not proof every byte reached the server. A positive ACK prefix
is not committed durability in PostgreSQL before COMMIT. A rejected response is
separate from transport loss. Quarantine/socket drop is not ROLLBACK ACK. Progress
is stored before each await, so dropping a pending future leaves its unknown
boundary and counts inspectable; it cannot restore a handle or execute anything.
ROLLBACK cleanup on failure is allowed even after current authority changes, within
the original deadline only. It sends no new DDL and claims rollback only on ACK.
No fresh timeout is created. ACK completion plus a later invalid current record is
reported separately, never converted into valid success.

Real MySQL COPY ALTER can report copied rows: the dedicated fixture measures
rows_affected=1 for each of two successful steps, as well as ordinary zero-count
ALTERs. Neither exact0 nor exact1 gates DDL completion. The fixture sets
`old_alter_table=ON` only on its dedicated session before capture; no production
API for that control or algorithm callback is added.

There is no general exactly-once, universal DDL rollback, lock-free, data-loss-free,
full effects or physical identity guarantee. DDL can block workloads. Borrowed and
external transactions, reconnection, role switching and arbitrary engines are not
supported. Independent observers/tests do not become a product executor API.

## Output and compatibility

All sensitive records have redacted Debug, no Display/serde. Closed errors have no
external error text or source chain. Driver calls, polling and drop use the existing
quiet-driver wrapper. Adapter/current-source implementations still own their own
logging and panic behavior. Default output carries no SQL/names/values/role/subject/
configuration/opaque bytes. Existing sensitive artifact export alone remains opt-in,
new-file/no-overwrite, protected Linux creation, CI/MCP refusal and Windows
UnsupportedOs; no new export/input route is added. Core remains dependency-free;
Rust1.94/edition2024/publish=false/pins/lock/features, CRUD gate, source wire,
manifest provenance, command/owned/borrowed/catalog/CLI/MCP contracts remain intact.
All 24 full runtime_pending markers remain pending. This PR neither accepts the
whole Issue12 nor closes it. See [finite PR3 evidence](migration-apply-v1-evidence.md).
