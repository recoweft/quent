# QT-AI-12 PR2: explicit bounded request observation

This is application-only opt-in on the existing runtime sessions. It adds no
feature or dependency: selected `postgres`/`mysql` plus `runtime-tokio` expose the
observation types. Core and default facade/CLI remain driver/runtime/serde free.
Rust1.94, edition2024, publish=false, pins and lock are unchanged. PR1
[PerformancePlan](performance-plan-v1.md) remains separate Partial/Estimated
evidence; no observation becomes authorization, an estimate or a planner.

```rust,ignore
let budget = ObservationBudget::new(16, 32, Duration::from_secs(10), 3)?;
let mut request = RequestCollector::new(&current, budget);
{
    let mut scope = session.observe(&mut request)?;
    let handle = scope.validate(&current, &operation, Duration::from_secs(1))?;
    let result = scope.execute_validated(&current, &operation, handle).await?;
    // Read the complete bounded result through existing explicit sensitive access.
}
let evidence = request.finish();
let public_json = evidence.public_view().to_json();
```

## Ownership, scope and finite coverage

The application owns one non-Clone/non-Copy/non-serde collector for its authenticated
request. It binds the **whole immutable DatabaseContext**, including complete
settings/policy/schema contents and generations, subject, native tenant and exact
target/dialect. Attachment to another snapshot refuses before execution. Multiple
sessions with that exact snapshot may attach sequentially and share the same
cumulative counters. Another newly constructed collector is a different request;
construction cannot authenticate the application's real request boundary. No
cross-request durable limit, DB exactly-once or physical routing authentication is
claimed. The application must not deliberately create a new request to evade its
own higher-level policy or resupply an obsolete current snapshot.

`ObservedPostgresSession`/`ObservedMysqlSession` and their borrowed counterparts
exclusively borrow the concrete session and `&mut RequestCollector`. There is no
Clone, Deref/DerefMut, AsMut, raw/session getter, callback or backend trait. The
caller cannot swap an unobserved session into a scope or run the original session
while the scope is live. A scope detaches on Drop. `finish` closes the request and moves its private shapes into immutable final evidence; even a retained attachment cannot submit after it. Forgetting a scope leaves the
private enforcement attached, rather than silently removing it. A forgotten pending
future remains Pending/incomplete; reattachment refuses. Concurrent scopes sharing
one request are compile-time refused; the attempt-entry check also refuses a
second pending operation through deliberately forgotten scopes before begin/send; sequential accumulation and async repoll are
supported. No mutex guard crosses a SQLx await, no ambient/thread-local/global
request/tenant context is used. The private mutex is for lifecycle/evidence mutation,
not a public concurrency capability.

| API/path | PG16.14 / MySQL8.4.8 with collector selected |
| --- | --- |
| Ordinary permitted SELECT/COUNT/UPDATE/physical DELETE | Supported through original validation and common final dispatcher |
| Delayed `validate`/`execute_validated` | Supported; handles retain original session/input/expiry checks; no new handle authority |
| Owned `validate_atomic`/`execute_atomic` | Supported existing single-business-operation Tx, exact expectation, finite version and PG RETURNING; complete inspection before completion |
| Borrowed adapter `observe` and ordinary/delayed execute | Supported; outcome BorrowedUnconfirmed, no completion ownership |
| Borrowed atomic execution | No execute method; existing validation refuses Unsupported; no atomic promise |
| OperationSpec `execute_compiled_read` | Supported on owned scope with `operation-spec`; same sealed record/gate, no new compiler |
| Generated repository entry requiring concrete session; generated decoder wrapper | No scope conversion; compile-time unsupported. Application can use supported compiled-read operation explicitly; no generated-result-consumption claim |
| Authorized candidate/acquisition/execute/atomic | Not exposed by scopes; compile-time refusal before provider/begin/send. Attached raw-session retention after scope forgetting additionally refuses at candidate/acquire/execute entry |
| Command execute/lookup/bundle including replay | Not exposed; compile-time refusal before ledger/begin; retained attached session additionally refuses at common command entry |
| PR1 performance acquisition | Not exposed; compile-time refusal before version/EXPLAIN. Attached session additionally refuses before first I/O |
| Plan/prepare/compile/review/validation | Remain DB/provider0. Pure validation refusal is returned directly; it is not a submitted-query entry |
| Arbitrary SQL/raw executor/backend callback/view restoration | No entry, compile-time refusal |
| Connect/handshake/driver initialization/protocol internals | Outside request scope, unobserved, not counted as 0 total I/O |
| Independent manifest/catalog/preflight/migration connections | Separate APIs/connections outside this scope; unobserved, no request budget claim |
| External authorization provider | Unsupported in scope, not observed as an external service; no implicit invocation |
| CLI/MCP/wire/export/tracing/SARIF | No new input, tool, exporter or emitter; public output only via explicit view |

This table does not expand any existing dialect/native/policy/tenant/read-only/risk
capability. MySQL RETURNING, ordinary INSERT/High/Unknown, raw/batch/upsert/join and
other pre-existing refusals remain. All corresponding existing APIs without a
collector keep their existing semantics. Explicitly using a separate session outside
an application-chosen request scope is outside this local API boundary.

## Budget, counts and lifecycle

The delivered specification consultation selects A: budget is a maximum of
**corresponding non-Tx-control SQL submission attempts** inside the explicit request,
not logical calls, rows, packets, all communications or server statements. Each
supported business statement consumes one unit. The supported subset has no
auxiliary/EXPLAIN/ledger/audit/outbox/provider calls; their scoped acquisition APIs
are refused, never silently excluded from a supported operation. If those paths
become supported, every actual auxiliary SQL must consume a separate unit with an
exclusive classification. Their existing unobserved counts must not be attributed
to this collector.

Shape preparation happens after finite native binding and **before** the original
final current/session/operation/permit/expiry checks. After these exact checks the
common final submission boundary performs check-and-charge once, immediately before
the existing closed SQLx execute/fetch. There is no application await/callback in
between. A future unpolled or refused by an earlier gate consumes no SQL unit.
Repoll resumes the same async attempt and cannot charge twice. A rejected next SQL
has dispatch0; earlier sends remain charged. Driver errors, cancellation, timeout,
rollback or unknown completion never refund a charged attempt. A supported atomic
operation may already have begun its Tx before business budget refusal; necessary
rollback is still attempted. This is not a guarantee of begin0/all-traffic0.

Limits: submissions 0..4096, entries 1..128, repeat threshold 2..128, measurement
ceiling >0 and <=300s per operation. The entry cap also bounds successful sends to
128, even if a larger submission budget was supplied. Capacity failure refuses
before another operation starts, marks capacity_exceeded/incomplete/closed and
never drops an entry while pretending full observation. No truncation success,
counter reset, restore, refund or retry API exists. A gate/driver/result/expectation
failure, duration ceiling excess, interrupted attempt, unsupported retained entry,
or budget/capacity rejection closes the request; remaining budget cannot be reused.
New explicitly constructed requests have their own state. A pure validation failure
before entering the dispatcher creates no entry, sends nothing and consumes no unit.

Entries count attempted observed **operations**, including a final-gate refusal;
`submissions` counts only charged SQL attempts. For the finite supported subset each
entry has zero or one submission. Native rows/semantic bytes are separate consumption
counts, never estimates, affected-value changes, wire bytes or a disclosure of values.
`decoded_rows` counts complete successfully decoded rows (COUNT has one native
8-byte scalar); `decoded_bytes` follows the existing semantic size accounting.
A later-row/column failure records only prior fully accepted rows and Partial, not
a truncated success. A native write response with no rows has Complete consumption
but supplies no row-value observation. Existing result limits are unchanged.

| Observation | Meaning |
| --- | --- |
| Succeeded | Existing executor returned success; full supported driver-result consumption completed |
| Refused | No business submission/begin occurred in this attempt; fixed typed error returned |
| Failed | Submitted business SQL or began owned Tx but execution/expectation/completion failed |
| Interrupted | Polled future was dropped without returning a completion; no server cancellation proof |
| Pending | Future has not completed and may have been forgotten; view is incomplete |
| NotObserved / Partial / Complete | No accepted result yet / some accepted results before failure / complete supported driver consumption |

An entry starts on the first poll of the common dispatcher (owned atomic: before
entry checks/begin), after pure validation/compilation. The `Instant::now` interval
ends after the driver's complete bounded consumption/decode and executor processing
(owned atomic additionally exact expectation and commit/rollback await), or at
Attempt Drop. It includes gate, native binding, private shape comparison, waits,
scheduling and result inspection. It excludes connection, pre-execution validation,
provider (unsupported), application processing of the already returned owned vector,
subsequent application Tx completion and idle time between calls. It is not isolated
server execution time or wall-clock time. Integer microseconds are rounded down;
`Exact` means the interval was below the ceiling and not clipped, not nanosecond
precision. CeilingExceeded publishes the bounded ceiling value and incomplete scope,
closes future use, and **does not** cancel server work or undo an already sent write
or an acknowledged commit. No timeout/maximum-latency guarantee is introduced.

BEGIN/COMMIT/ROLLBACK do not consume SQL budget. Each entry separately records the
library's explicit begin/commit/rollback attempts and ACKs (0/1 each) and finite Tx
outcome. None means ordinary execution made no owned Tx; it does not prove an
autocommit write did not take effect. BorrowedUnconfirmed never becomes Committed
because an external caller later commits. Owned NotStarted becomes Unknown when
begin starts; only successful completion ACK becomes Committed/RolledBack. Begin,
commit or rollback failure/lost reply/cancel stays Unknown and existing quarantine
is preserved. SQLx Drop's best-effort rollback is not an observed explicit attempt
or an ACK. Commit failure does not add a retry or fabricate confirmed rollback.
Expected-row failure never commits and cannot be waived by the collector.

## Finite repeat evidence and confidentiality

The private Shape is derived from immutable final record metadata: exact table/path/
alias, family, ordered projection/aliases, AND/OR/NOT/comparison tree and column
identity, IN arity, native types/NULL distinction, ordered writes/order/version and
limit presence. Literal values (including limit values) are omitted; types/NULL and
structure remain. All exact names are retained privately for bounded equality.
No SQL replacement/normalization/hash is used; there is no hash collision equality
claim. Full request snapshot binding prevents target/tenant/subject conflation.
Maximum 128 shapes use existing bounded accepted IR, never arbitrary caller trees.
No SQL renderer or independent planner/execution restoration is added.

repeat_count includes the current submitted statement plus prior exactly matching
shapes in this request. Only the current entry's threshold flag becomes true when
the count reaches the configured threshold. `n_plus_one_suspected` is a finite
repeat suspicion, not a diagnosed N+1 cause, performance regression, authorization,
safety, improvement or absence proof. Failed/canceled/dropped/overflowed observations
retain incomplete state; false flags there do not mean no repetition outside the
observed subset. New request/different projection/predicate/target controls do not
inherit repetition evidence.

RequestCollector and QueryObservation have fixed redacted Debug, no Display/serde/
raw getter/exporter. The latter retains private context/shape evidence until dropped.
ObservationView JSON/Debug/Display contains only fixed kind/version/enums/booleans
and bounded numeric counts/durations. No SQL/value/hash/identifier/tenant/subject/
target/table/column/index/alias/path, opaque ID or arbitrary error cause is included.
New errors DispatchBudget/ObservationLimit/ObservationClosed are closed enums with
fixed Display and no source; their existing diagnostic projection uses ResultLimit.
The public view is output-only: no Deserialize or conversion to collector, permit,
ValidatedOperation or context. JSON is never execution authority. Existing driver
statement/slow logging disablement and private poll/drop suppression, sensitive
local-details restrictions, manifest/fingerprint/wire and PR1 limitations remain.

## Evidence and limits

C21 is central; C01/C06/C08/C11-C19/C23/C24 keep their relevant shared-gate,
request/target/tenant/ownership/expiry/Tx/result/confidentiality/nonrestoration
regressions. C17/C18 command tests remain existing unobserved regressions, not new
collector command support. C23 adds no MCP implementation or external delivery.
All 24 full runtime_pending markers remain unchanged.

Dedicated public-scope tests exercise zero/1/2 limits and next-dispatch0, no-poll0,
exact SQL/native args and recorder events with/without collector, request separation,
exact-context refusals, capacity, repeat positive/negative, full read consumption,
owned commit/expectation rollback/post-begin budget rollback and caller-owned Tx.
An independent SQLx observer compares data and column/type/nullability/order catalog
before/after. Existing real server-observed atomic lock wait/drop/timeout/termination
controls now also inspect request counts and Unknown/not-consumed state; original
assertions and deterministic authorized-cancel controls are retained. Begin/commit/
rollback failure and lost-successful-ACK tests use the existing **artificial seam**,
not a new claim of live protocol ACK loss. Existing command proxy tests remain
separate evidence for their unobserved paths. A 256-row partial decode is observed
through the production result-limit error. No cfg(test) recorder is a full wire or
server statement monitor.

Compiler controls use the real public scope/read/atomic/JSON APIs with a compiling
control, then diagnostic-checked refusal of serde/Clone, request/view/permit/handle
restoration, consumed evidence, raw scope replacement/private access and parallel
request scopes. Independent public key/value allowlists have a canary detection
positive control. Limits on protocol-buffer allocation, external logs, physical
routing, server scheduling and external current stores remain application duties.
Windows live DB, TLS, arbitrary network faults and arbitrary schema semantics are
unverified by WSL tests. Hosted per-job results must be checked at the final head.

Performance comparison is an observation, not acceptance of improvement. Both
variants use the same already connected session, SQL/native args/settings and
unchanged data: ten complete SELECTs alternating with ten exact1 committed owned
updates per measured round. Collector construction is outside the monotonic span;
scope attachment/detachment and full execute/validation/consumption/Tx completion
are inside. Four paired warmup rounds precede nine paired measurement rounds;
order alternates baseline-first/collector-first. Each measured variant round has
20 business submissions, ten begin ACKs and ten commit ACKs; collector adds no
auxiliary/provider/EXPLAIN I/O. Raw ns, median/min/max/MAD are printed by each real
DB test and recorded with final commands/results in the PR/RelayWeft report.
Debug-build assertion/recorder cost, CI noise, caches and this small local workload
limit interpretation. No predeclared improvement or latency bound, release-build
benchmark claim or extrapolation to Windows/other workloads is made.

[Finite local evidence, case counts and raw performance measurements](query-observation-v1-evidence.md) records the WSL fixture observations separately from hosted exact-head results.
