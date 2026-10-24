# QT-AI-03 planned PR3: finite atomic results

The owned API is intentionally a **single business operation transaction**:
`session.validate_atomic(current, operation, ExpectedRows::exactly(n)?, ttl)` then
`session.execute_atomic(current, operation, expected, handle).await`.
Quent owns begin, the sole business submission, complete bounded result inspection,
expectation comparison, and commit/rollback. There is no exposed owned transaction
executor, separate commit, callback, savepoint, multi-operation builder or raw SQL.
The transaction cannot survive a validation failure in a caller-committable state.
The session can begin a new explicit transaction after confirmed completion.
No provider, command ledger, automatic retry, replacement ID or exactly-once promise
is introduced. Authorization and authentic physical schema/context remain app duties.

## Bindings and the common gate

Validation is DB/provider-free. The non-Clone/Copy/serde atomic handle privately owns
PR2's exact sealed preparation, full immutable settings/policy/schema/target/tenant/
subject snapshots, connection allocation identity and exclusive monotonic expiry,
plus the exact bounded expectation (0..256). Execution consumes it on future
construction. Changing expectation, operation (including version/projection), context
or session refuses with zero business sends. No public getters or restoration path.

After the begin await, the same closed dispatcher rechecks identity, full current
snapshot, exact bounded operation and expiry at entry and again after finite native
encoding immediately before submission. The real SQLx transaction exclusively
borrows the same session connection; its context is cloned without mutation.
There is no separately exposed owned-Tx handle that can target another transaction.
Existing delayed session handles remain bound to their original connection identity.

`BorrowedPostgresTransaction::new(&mut sqlx_tx, context)` and the MySQL equivalent
hold the exclusive connection borrow for the adapter lifetime and have fresh private
identities. Handles from sessions, other adapters/transactions, or adapters that were
dropped cannot be executed on a replacement adapter, including one over the same
transaction. This part is runtime identity refusal; it is not a claim that the Rust
handle type itself has a generative transaction lifetime. Rust prevents ending or
replacing the underlying transaction while the adapter/future borrow is still used.
The live tests pair both identity refusals and valid execution controls.

All operations use the existing shared Strict preparation/risk/capability gate,
sealed final record, finite native binder and decoder. High/Destructive/Blocked,
Partial/Unknown, INSERT, broad writes, unsupported schema semantics and soft-delete
Partial still refuse. Version presence never supplies the ordinary key proof.
No application-authenticity, external DDL, RLS/trigger or routing verification is added.

## Final IR and dialect semantics

`Draft::optimistic_version(column, expected)` is UPDATE-only. The separate immutable
IR field means equality against expected and assignment of its checked successor.
The original predicate must independently establish permitted scope after ordinary
tenant/policy preparation. Even an asserted unique version column cannot replace it.
Domain: nonnullable signed I16/I32/I64 on PG; I8/I16/I32/I64 on MySQL. Negative values
are valid. Overflow, NULL, unsigned/bool/text, duplicate version/data assignment,
wrong schema/type/column, tenant, readonly/generated or protected PII refuse before
business submission. No expression, column inference or arbitrary arithmetic API.
A versioned atomic operation requires expectation one. Zero matches is `Conflict`
(including missing ordinary key); multiple matches is `ExpectationMismatch`.

The sole finalizer binds successor, original predicate values, policy guards and
expected version, and renders the final SQL once. Metadata retains the exact original
operation, ordinary predicate, separate `sensitive_version`, complete ordered write
data including successor, and final projection. Inspection conservatively uses the
ordinary predicate for scope while checking schema coverage of version/returned
columns too. No SQL text is changed after finalization or reconstructed by adapters.

`Draft::returning(Vec<Projection>)` supports PostgreSQL UPDATE and physical DELETE
only. Empty means absent. The finite explicit projection is resolved, quoted and
captured in the same finalize as SQL/args. Names, native types, nullability, schema
coverage, PII/column policies and existing 256-column/argument/SQL budgets apply.
Readonly/tenant columns can be returned if other policies allow reading them;
readonly/generated/tenant writes still refuse. Duplicate names refuse. MySQL,
SELECT/COUNT/INSERT RETURNING and write order/limit/alias refuse before send. There
is no separate SELECT fallback. INSERT/High are still unsupported execution.

| Result observation | Meaning used by exact expectation |
| --- | --- |
| PostgreSQL UPDATE/DELETE without RETURNING | SQLx command rows affected, matched rows under asserted ordinary-table semantics; unchanged UPDATE is one |
| PostgreSQL UPDATE/DELETE RETURNING | All returned rows/columns consumed and decoded; cardinality of the complete returned set, including zero |
| MySQL UPDATE/DELETE | `MysqlFoundRows`: SQLx 0.9.0 enables CLIENT_FOUND_ROWS; unchanged matched UPDATE is one |
| MySQL changed-row count / upsert / unsupported native semantics | No reinterpretation or success fallback; unsupported |

Matching one is not evidence that one value changed. Live unchanged UPDATE compares
both the native driver count and unchanged observer value. MySQL's default client
changed-row semantics cannot be substituted for this pinned FOUND_ROWS connection.
Guarantees assume accurate ordinary transactional tables (InnoDB in the fixture),
complete key/schema facts and no unsupported side effects. Autocommit observation
alone is not an atomic expectation result.

All RETURNING rows and columns must pass the finite decoder before commit. Existing
limits are 256 rows, 4096 bytes per text and 1 MiB semantic result payload. A later-row
NULL, later-column native mismatch or any result limit fails the entire operation;
no first-row exactly-one evidence or truncated success. On failure the stream is
dropped and explicit SQLx rollback is awaited; decode failure does not require
continuing to allocate/decode the remaining failing result. Driver buffering/server
work is outside the semantic retained-value budget.

## Outcomes and lifecycle

`ExecutionResult::state()` distinguishes normal `Observed`, borrowed `Uncommitted`
and successfully checked/committed `Committed`. Only explicit sensitive accessors
return values. Atomic errors expose fixed `cause()` and `disposition()` classifications,
no driver source chain or arbitrary text. New handles, expectations, adapters and
results have redacted Debug; errors contain fixed enums only. No public JSON/serde
or additional execution restoration path exists.

| Stage | Business sends | Return / availability |
| --- | --- | --- |
| Validation/entry mismatch or expired | 0 | `NotStarted`; no owned begin; session unchanged |
| Atomic future constructed, never polled, dropped | 0 | no return; handle consumed; session available |
| Begin attempt fails/canceled | 0 | failure is conservatively `OutcomeUnknown`, or no return on drop; session unavailable |
| After begin, pre-send expiry/binding/encoding failure | 0 | explicit rollback awaited; `RolledBack` only on acknowledged success |
| Complete native write/decode, exact expectation succeeds | 1 | commit awaited once; only success returns `Committed`; session available |
| Version/count/decode/budget/driver failure | at most 1 | no commit; rollback awaited once; `RolledBack` if acknowledged, otherwise `OutcomeUnknown` |
| Commit error/lost reply | 1 | `OutcomeUnknown`; session unavailable; never retry |
| Polled atomic future canceled/timed out/dropped | at most 1 | no AtomicFailure value can be returned by a destroyed future; session unavailable and outcome unconfirmed |
| Attempt to reuse quarantined session | 0 additional | `SessionUnavailable`; prior uncertainty remains |

SQLx official 0.9.0 sources were inspected locally and compared byte-for-byte with
cached registry archives whose SHA256 matches Cargo.lock: `sqlx-core/src/transaction.rs`,
`sqlx-postgres/src/transaction.rs`, `sqlx-mysql/src/transaction.rs` and MySQL
`connection/stream.rs`. Transaction::begin sets open before awaiting; commit/rollback
consume self and clear open only after success. Drop calls start_rollback, which
queues work; it does not await or prove rollback. Both drivers track transaction
depth and queue rollback on Drop. A commit failure can thus cause best-effort
rollback queueing, without establishing whether the server already committed.
No asynchronous Drop action is reported as confirmed rollback. Fixed begin/commit/
rollback statements are driver transaction control, never arbitrary business SQL.

Borrowed adapters return only `Uncommitted`; `validate_atomic` is fixed Unsupported,
and no borrowed execute_atomic/commit/rollback API exists. Dropping a borrowed adapter
does not drop the application's SQLx transaction. The caller may continue/commit
(or rollback) after ordinary success, refusal or error, subject to actual server
transaction state. Quent cannot mark it rollback-only or enforce atomic expectations.
The caller owns this decision and must not interpret a detected error as automatic
rollback. Context-to-connection correspondence and the external SQLx connection's
statement-log configuration are also caller duties; disable SQLx statement/slow
logging when constructing external connections. Owned sessions disable both.

## Evidence and its limits

`execution::tests::live_postgres/live_mysql` extend the production-path PR1/PR2 tests
on dedicated digest-pinned containers and limited roles. The production cfg(test)
recorder observes begin/begun/send/full decode/expectation/commit/committed or explicit
rollback/rolled_back. A shared finite trace preserves submission evidence after
future cancellation. This is separate from the scripted support recorder.
Independent SQLx observer connections check committed values and absence of writes
on confirmed rollback. Only disposable tables/roles/schemas/connections are affected.

* Both engines: success and unchanged matched=1, expected zero/one/two vs actual
  zero/one/multiple, version conflict and checked successor; expiry after begin;
  operation/context/expectation mismatch; borrowed uncommitted result and unsupported
  atomic validation with zero sends/no automatic completion; caller commit after
  refusal; cross-session/cross-adapter/ended-Tx identity rejection.
* PostgreSQL: UPDATE/DELETE RETURNING, version + RETURNING, all rows/columns, later-row
  NULL failure (one whole row decoded first), later-column native mismatch, text/row/
  aggregate result budgets, PII projection refusal and allowed readonly/tenant projection.
* MySQL: unsupported RETURNING refuses zero sends, including no begin.
* Multiple-row counterexamples deliberately remove the real primary key in this
  disposable fixture while retaining an inaccurate asserted key. The ordinary gate
  is never bypassed. This explicitly demonstrates the application assertion limit.
* Actual server row-lock waits for the exact business connection are observed before
  owned drop/timeout; only that connection is terminated for the disconnect experiment.
  No commit/confirmed rollback is claimed on those paths; observer values remain
  unchanged after dedicated setup cleanup, and the session is quarantined.
* Separate finite cfg(test) faults replace begin/commit/rollback completion with a
  fixed driver error, or discard an actual successful commit/rollback acknowledgement.
  The ordinary classification branch is shared. These prove error handling and
  observer differences (lost successful commit changes data), not actual wire loss.
* Core tests check exact SQL/args/metadata/version/projection and independent original
  scope proof. The compiling API controls and 29 diagnostic-checked negative programs
  cover consumed atomic/ordinary handles, serde/Clone/Copy boundaries, async input/
  session borrows, borrowed lifetime/commit borrow and borrowed atomic API absence.

Real network loss exactly at commit/rollback acknowledgement, cancellation during
those specific protocol stages, and begin/commit/rollback network timeouts are
unverified: this fixture has no deterministic network proxy/server completion hook.
Finite injection is reported separately. TLS handshake and local Windows execution
are unverified; hosted CI results are recorded separately. No unsupported DB/server,
trigger/RLS/nontransactional table, provider authorization or QT-AI-10 ledger recovery
is claimed. These do not become supported by passing Linux local tests.

No production dependency, lockfile, feature default, Rust 1.94.0/edition2024/publish=false
setting or CI job is changed. Core/default remain driver/runtime-free. The all-24
`runtime_pending` fixture markers remain unchanged: this is finite C16 evidence with
C11–14 inheritance/ownership and C02/C06/C07/C10/C15 regressions, not full-case completion.
Measured commands, exact head, server settings, skips/failures and hosted CI status
belong to the PR/execution report; source does not assert its own future head SHA.

## Local verification for this revision

WSL2 Ubuntu 24.04.5; rustc/cargo 1.94.0. Final full live workspace run: 91 tests
passed, zero failed/ignored/filtered (original 87 plus four Core IR tests; the
compile harness's 29 negative programs and two controls are inside one Rust test).
`bash scripts/test-dispatch.sh cargo test --workspace --all-features --locked --offline`
passed and explicitly verified dedicated container/volume cleanup. The standalone
live suite also passed during development. No missing environment or ignore was
counted as DB acceptance. `--skip live_` is the explicit DB-free subset: 88 passed,
three excluded (two live engine suites plus the pre-existing Core context test whose
name contains live); those exclusions are not live evidence.

Also passed: fmt check, workspace default/all-feature build, default tests,
warning-denied all-target/all-feature clippy and rustdoc, all 16 feature subsets,
Core and default facade normal dependency trees. Cargo.lock/manifests/toolchain,
fixtures and required live/feature CI jobs are unchanged.

| Dedicated engine | Digest and measured settings |
| --- | --- |
| PostgreSQL 16.14, x86_64 Alpine | `postgres@sha256:16bc17c64a573ef34162af9298258d1aec548232985b33ed7b1eac33ba35c229`; UTC, read committed, UTF8, en_US.utf8 |
| MySQL 8.4.8 | `mysql@sha256:2952e3be7807f06fc18de50b3ea1a632d5c70d63482ff7d7376fe3aa8999babf`; +00:00, REPEATABLE-READ, utf8mb4/utf8mb4_bin; ONLY_FULL_GROUP_BY, STRICT_TRANS_TABLES, NO_ZERO_IN_DATE, NO_ZERO_DATE, ERROR_FOR_DIVISION_BY_ZERO, NO_ENGINE_SUBSTITUTION |

Roles are created only in these new containers. Business/observer connections use
`quent_test`, with schema/database permissions and MySQL SELECT only on the two
performance_schema tables needed for exact lock-wait observation. No existing DB,
host global configuration or unrelated connection is changed. Hosted CI status,
PR URL and final head must be checked in the execution report; local checks are not
controller independent retests.

## QT-AI-04 PR1 output boundary

[Public views v1](public-views-v1.md) adds output-only finite projections of the
current retained record/inspection/preparation and fixed errors/result states.
Historical statements above deferring public views describe their named baseline.
No public JSON or copied display classification is execution authority. Sensitive
getters and the complete private gate remain unchanged; CLI/tracing/export remain
with PR2/owning issues. All 24 full runtime_pending markers remain.
