Current QT-AI-09 PR2: [scoped final execution permits](authorization-gate-v1.md)
connect explicit request/session-bound acquisition to this same common dispatcher.
Ordinary paths retain provider0 and High refusal. PR1 absence statements below are
historical; its unbound acquisition remains non-executing.

Current PR3 additions: [finite owned atomic results, version/RETURNING IR and borrowed Tx boundaries](atomic-results-v1.md). Earlier PR-specific absence statements below describe their historical baseline.

# QT-AI-03 planned PR2: consumed validation

`PostgresSession` and `MysqlSession` expose synchronous
`validate(&current, &operation, ttl: Duration) -> Result<ValidatedOperation, ExecutionError>`
and `execute_validated(&mut self, &current, &operation, handle)`.
Validation is DB/provider-free. It uses PR1's finite Strict preparation, sole Core
finalizer, inspection and refusal gate. The private handle owns that exact
PolicyPreparation: original immutable operation, final SQL/ordered native args,
metadata/original missing-WHERE bit, exact full context and schema evidence.
It additionally retains an allocation identity unique to the owned connection
lifetime and a monotonic deadline. No public getters, serialization, Clone, Copy,
Display, executor restoration or retry alias exist. Debug is a constant redaction.
This is structural validation, never business authorization/permit/commit evidence.

Immediate `execute` calls this same validation and consuming dispatcher. There is
no second renderer, binder, planner or bypass using Plan/public output/source.
Both APIs refuse High/Destructive/Blocked, incomplete/Unknown and INSERT exactly
as PR1 does. Low/Medium Complete remains conditional on the documented supported
semantics and application authorization. PR3 transactions and QT-AI-09 providers
are absent; batch/upsert/codegen/public JSON/MCP are outside this change.

## Current input and connection binding

Both the entry and final pre-send gate compare the session allocation identity,
full bound/current/prepared context, and original/current operation. Context equality
includes target/routing/catalog/dialect, settings/policy/schema identities AND
independent generations AND complete contents, tenant type/value and subject.
Same nominal context on another connection, including a newly connected session,
fails. The Arc identity is private and keeps its allocation alive while any handle
exists, preventing identity reuse after dropping its original session. It grants no
shared execution access. The connection is owned privately with no replacement API.

`Operation::same_bounded_input` first checks both borrowed inputs with the existing
finite iterative budget walk, then compares their complete tree/values/types/NULL/
Missing/projection/order/limit/write-pair order. Oversized current input refuses
before recursive Eq/clone/render. No new current Plan is constructed. Caller-side
construction/allocation/drop of arbitrary drafts is outside this bound. Immutable
context constructors already bound their contents. No labels, hash or reason stand
in for exact content. Mismatch sends zero and consumes the handle; a new explicit
validation is required. Rust ownership prevents reusing one handle, not constructing
multiple new handles, executing twice intentionally, or database exactly-once.

The app must supply authentic current snapshots and correct context-to-physical-DB
correspondence. Supplying the same obsolete snapshot twice, inaccurate schema/table
semantics, external DDL, wrong routing, triggers/RLS or server replacement is not
made authentic by comparison. No implicit catalog/provider I/O is added.

## Deadline and async boundaries

TTL is an explicit Duration: `0 < ttl <= 300 seconds`. Zero, larger durations and
checked-add overflow fail closed (`InvalidTtl`). Expiry is exclusive: now less than
deadline passes; equal or later returns `Expired`. The clock is std Instant, private
and monotonic; deadline starts before validation work, so preparation consumes TTL.
Plan/compile/inspect remain independent of time. Validation may consume its TTL;
execution always checks again. Immediate execution explicitly validates with 300s.

At entry, bindings are checked before sampling current time. After native encoding,
the final gate checks all bindings again and THEN samples actual time before adapter
submission. No production application await exists between encoding/final gate/send.
Connect is separate explicit I/O before validation; an existing owned connection is
used, so there is no pool acquisition await. The exclusive `&mut session` and immutable
borrowed current inputs span the SQLx await, preventing application replacement.
SQLx driver network/internal scheduling awaits occur after the library submission
boundary; this is not a guarantee about the time a server executes a packet.

The test-only final barrier/clock demonstrates final-gate expiry after an await,
including expiry equal/after and a successful one-nanosecond-before case. These hooks
are absent from the product API. There is no provider await or external decision;
no provider revalidation guarantee is claimed. Any future application/provider/pool
await before submission must recheck all required current bindings and sample time
AFTER that await and after bounded comparisons, immediately before send.

## Consumption and session stages

| Stage | Dispatch observation | Handle | Session / outcome |
| --- | --- | --- | --- |
| Validation refusal/invalid TTL | 0 | No handle created | Existing availability unchanged |
| Future constructed, never polled, then dropped | 0 | Moved and destroyed | Available |
| Polled pre-send refusal/expiry/encode failure | 0 | Consumed; never returned in Result | Available |
| Polled pre-send test barrier canceled | 0 | Consumed | Available; product has no such await today |
| Submission begun, then future dropped/canceled/timed out | One submission, no retry | Consumed; cancellation itself returns no Result | Unavailable; write outcome unconfirmed |
| Driver read/write/decode/result-budget error after submission | One submission, no retry | Consumed | Unavailable; write driver error is OutcomeUnknown |
| Successful complete consumption | One submission | Consumed | Available; affected count is an observation |
| Attempt to use unavailable session | 0 additional | Supplied handle consumed | Still unavailable |

A timeout wrapper reports elapsed time, not rollback/commit. No failed handle is
returned, no new operation ID is assigned, and no business rollback is confirmed.
Dropping the session does not turn canceled writes into confirmed failure. Explicit
rollback in the test setup concerns only its locking transaction. PR3 owns atomic
results/transactions; real connection-loss injection is not added/claimed here.

## Finite evidence and boundaries

The live suites use the production dispatcher recorder (after final checks and
encoding, before SQLx), not the scripted support recorder. Each dialect observes
exact retained-record sends twice; value/tree/type/NULL/Missing/projection/order/
limit/family/oversized/deep current-input mismatches, all three independent generations,
same-generation schema/policy content, identities/target/tenant/subject/dialect,
other sessions/reconnection, expiry and unpolled/pre-send cancel yield zero sends.
A separate pure test checks write-pair ordering and bounded comparison. Validation
refuses unsupported/high/blocked/unknown without sending. PR1 native/tenant/count/
write/result-limit/redaction/no-retry tests continue through the identical path.

For real cancellation/timeout, dedicated raw setup opens a transaction and holds a
row lock. A permitted business UPDATE is polled while an independent observer checks
PG `pg_stat_activity` Lock waits / MySQL `performance_schema.data_lock_waits`
joined to `threads`, restricted to the exact business connection ID. An explicit
test-only connection-ID query is setup, outside the dispatcher recorder.
Observation must occur within 10 seconds; no timing-only sleep proves submission.
Then drop or timeout (after observing the wait) consumes one dispatcher send and
makes the session unavailable. This measures actual server waiting and library
session state, not business rollback or wire exactly-once. Each test explicitly
resets only its disposable fixture. Harness timeout is 240 seconds; EXIT cleanup
removes containers/anonymous volumes, and cleanup failure fails the command.
MySQL's dedicated role gains SELECT only on data_lock_waits and threads for this observation.
No existing DB, external service or global host configuration is changed.

`tests/owned_api.rs` compiles a positive real API control and checks expected Rust
error codes for 15 negative programs (reuse, future move/drop, failure branch,
Clone/Copy/serde, Debug/view/raw Plan/source/parse/private restoration, and current/context/session
replacement during the future borrow). It rejects
missing imports/dependencies as evidence. Existing eight PR1 adapter compile-fail
examples remain. The fixture uses only dev dependencies already pinned; production
and Core/default feature dependency trees, lockfile, MSRV/edition/publish flags are
unchanged. Std Instant/Arc add no dependency.

QT-C12/C13/C14 receive this finite evidence; C01/C02/C06/C07/C08/C10/C11/C15 retain
PR1 regressions. All 24 full runtime_pending markers remain, including authorization,
physical correspondence, Tx inheritance/atomicity, public view/JSON, broader cancellation
and command responsibilities. TLS handshake, Windows execution and real network-loss
fault injection are not established by local Linux tests. Measured commands, final
head, CI status and server settings belong to the PR/execution report; no source file
claims its own future commit hash.

## Local measurements

WSL2 Ubuntu 24.04.5, Rust 1.94.0: the complete workspace/all-feature run with
both dedicated DBs passed 87 Rust tests (PR1's 80 plus seven: one pure boundary
test, one diagnostic-checked compilation harness, five handle rustdoc examples).
The harness additionally checks 15 negative programs and a positive control.
Zero failed/ignored/filtered in the full live run. DB-free all-feature testing
explicitly uses `--skip live_`: 84 tests, three excluded (the two real DB tests
and Core's `c12_exact_target_and_context_missing_never_become_live_or_permission`).
These exclusions are not real DB acceptance. The full live command runs all three.

Commands: `cargo fmt --all -- --check`; `cargo build --workspace --locked --offline`;
`cargo test --workspace --locked --offline`; `cargo test --workspace --all-features
--locked --offline -- --skip live_`; `bash scripts/test-dispatch.sh cargo test
--workspace --all-features --locked --offline -- --nocapture`; warning-denied
workspace/all-target/all-feature clippy and rustdoc; `bash scripts/check-features.sh
--offline` (16 subsets); Core/default normal dependency trees. All passed locally.
The dedicated cleanup additionally verifies absence of exactly this run's containers
and anonymous volumes. The existing nonoptional live/feature CI jobs are preserved;
actual hosted CI results are reported separately.

## QT-AI-04 PR1 output boundary

[Public views v1](public-views-v1.md) adds output-only finite projections of the
current retained record/inspection/preparation and fixed errors/result states.
Historical statements above deferring public views describe their named baseline.
No public JSON or copied display classification is execution authority. Sensitive
getters and the complete private gate remain unchanged; CLI/tracing/export remain
with PR2/owning issues. All 24 full runtime_pending markers remain.
