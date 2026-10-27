# QT-AI-12 planned PR1: explicit estimated performance evidence

`quent/performance-plan` adds application-only APIs to the existing privately owned
`PostgresSession` / `MysqlSession` (also require the selected driver and
`runtime-tokio`). Core/default facade/default CLI remain driver/runtime/serde free.
There is no CLI/input wire, MCP/AI tool, collector, request dispatch budget, measured
latency, N+1 detector, new permission, source format or manifest fingerprint change.

## One retained record and explicit acquisition

`session.performance_request(current, operation, PerformanceBudget, ttl)` is a
pure opt-in using **the same** `validate` → common policy prepare → sole Core
finalizer → complete inspection gate as ordinary CRUD. It retains that private
immutable preparation, original operation, exact context, native ordered arguments,
metadata, session allocation identity and exclusive monotonic expiry. It refuses
non-SELECT after validation. Protected/tenant/schema/native/unknown/risk refusals
are unchanged. Supplying Plan, SQL, source, public JSON, validated/authorized
handles, callbacks, closures, executors or arbitrary EXPLAIN options is impossible.
Typed/Spec operations do not acquire an independent planner.

`session.acquire_performance(current, operation, request)` consumes the request.
The application must supply independent authentic current inputs. Full bounded
operation/context/subject/tenant/target/dialect/session comparisons precede a real
`Instant::now() >= expiry` check on entry, after the version await, after native
binding immediately before EXPLAIN, and after complete result consumption/parse
before return. A final real clock check follows the last full comparison. TTL
uses existing `0 < ttl <= 300s`; operation time budget includes auxiliary I/O,
EXPLAIN and parsing and is checked again after all comparisons. No hash is used
as equality. Immutable borrows and `&mut session` prevent application replacement
across awaits. No callback acquires the raw connection.

Each acquisition performs exactly one fixed auxiliary statement:

| Dialect | Version statement | Exact supported value | EXPLAIN prefix |
| --- | --- | --- | --- |
| PostgreSQL | `SELECT pg_catalog.current_setting('server_version_num')` | `160014` (16.14) | `EXPLAIN (FORMAT JSON) ` |
| MySQL | `SELECT VERSION()` | `8.4.8` | `EXPLAIN FORMAT=JSON ` |

Only the literal prefix is joined to the **unaltered** retained finalized SQL.
All original arguments, including typed native NULL and limit, use the existing
finite SQLx `try_bind` adapter in original order; no values enter SQL text. No
ANALYZE, arbitrary option/hint, prefix replacement or user SQL is accepted. No
business SELECT/CRUD is submitted to obtain results/elapsed time. SQLx connector
handshake/initialization is separate explicit `connect` I/O, outside acquisition
counts; Plan/prepare/compile/review remain DB/provider0, ordinary CRUD adds EXPLAIN0.
Acquisition itself starts no transaction and calls no authorization provider.

Once auxiliary I/O starts the session is quarantined until the entire operation
succeeds. Timeout, read/decode/metadata/budget/version refusal, expired return and
canceled polled futures leave it unavailable. An unpolled future sends nothing;
requests are consumed even when dropped/refused, with no retry or restored handle.
No result means no observation, never a partial success falsely labelled Fresh.
Quarantine/cancellation does not prove the server stopped optimizer work or released
locks. No commit/rollback/transaction guarantee is added.

## Finite coverage and result budgets

| Query/result capability | PostgreSQL 16.14 | MySQL 8.4.8 |
| --- | --- | --- |
| Common-gated single ordinary base-table SELECT, explicit projection, finite existing predicates/order/limit and supported native args | Candidate | Candidate |
| DML, COUNT, Raw, arbitrary function/SQL/hint, CTE/subquery/join, unknown effects/context/type/risk | Refuse; no EXPLAIN | Refuse; no EXPLAIN |
| Result structure | One JSON-array entry with exactly `Plan`; supported `Plans` tree | One `query_block`, `select_id=1`, single `table`; closed block/table keys |
| Operators | Seq Scan, Index Scan, Index Only Scan, Bitmap Heap Scan, Bitmap Index Scan, Limit, Aggregate, Result, Sort, BitmapAnd, BitmapOr | ALL, index, range, ref, const, eq_ref access; two exact impossible-WHERE messages |
| Estimated fields checked | startup/total cost (total >= startup), rows, width required | query cost, examined/produced rows, filtered 0..100 required on table blocks |
| Unknown node/access/version, missing required fields, nested loop/materialization/window/unsupported structure | Fixed refusal | Fixed refusal |
| Successful evidence coverage | **Partial**, Estimated; never Complete | **Partial**, Estimated; never Complete |
| Offline/runtime/live | Request creation DB-free; acquisition live only | Same |

This is structural coverage of a finite DB JSON subset, not all operators or all
query shapes within SELECT. PostgreSQL optional fields are bounded but not all
semantically interpreted; MySQL nested metadata is likewise not full optimizer
coverage. No result is safe/fast/authorized because it parsed. Unknown unsupported
structures are never converted into Complete. Impossible-WHERE is a structural
estimate (zero parsed nodes), not proof of actual business row count.

Budget constructor: result bytes 1..65,536; parsed plan nodes 1..64; timeout
0 < t <=10s. At most one row/one column is accepted and the successful stream is consumed to
end; a second row is a refusal sentinel. PG requires native JSON metadata, MySQL
finite JSON/text wire metadata before decoding bytes. NULL/wrong native metadata
refuse. Every raw result is bounded before copying into retained storage. The
production JSON first pass checks every token: 16 edges of depth, 2,048 values,
64 array children, 64 keys/object, keys <=128 bytes, strings <=4,096 bytes; duplicate
keys/multiple documents/malformed input refuse before `Value` construction. PG
plan tree additionally limits depth to 8 and 16 children/node. Numbers must be
finite nonnegative <=10^12; overflow, NaN, infinity, negative sentinel and invalid
plain MySQL numeric strings refuse. No truncation success. Application budgets
cannot bound SQLx incoming packet allocation or pre-decode protocol buffers, OS
scheduling, server optimizer resources or lock acquisition.

## Evidence and output are not authority

`PerformancePlan` privately retains the consumed request and sensitive raw JSON.
It is non-Clone/non-Copy/non-serde, has fixed redacted Debug, no Display/raw getter/
export/execution conversion. Raw EXPLAIN can contain SQL, names, predicates and
values; no raw-detail exporter is added. Existing protected local export remains
unchanged and does not accept this type. There is no output parser/restoration.
`PerformanceRequest` likewise cannot be restored/aliased or promoted from an old
handle/permit. New explicit requests remain possible; linear ownership is not DB
exactly-once. Evidence cannot be reused as execution authority after expiry or at
any earlier point.

`PerformanceView` is output only: fixed kind/version/state/Partial/Estimated,
parsed node count and fixed auxiliary1/EXPLAIN1/business0/provider0/Tx0 counts for
a successful observation. Its Debug/Display/JSON contain no SQL, values, tenant,
subject, target, table/column/index/alias/path, arbitrary driver/parser errors,
raw JSON, hashes or opaque metadata. Errors are closed enums, fixed Display and
no source chain. No tracing/SARIF/MCP integration accepts the new sensitive types.
Existing driver statement/slow logging and scoped poll/drop suppression apply.
No estimated rows/costs or observed timing are exported; costs across DBs are not
normalized or compared. Estimates are neither measurements nor proof of latency,
performance improvement, authorization, effect freedom or safe execution.

Non-ANALYZE EXPLAIN still contacts a DB, checks privileges, parses/compiles/plans,
may acquire metadata/relation locks and consume finite or substantial server work.
The supported contract requires accurate ordinary-base-table/no-effects/schema
assertions and builtin finite query semantics. It does not authenticate external
current stores, users, DSNs/physical routing, server identity or custom server
semantics. Uncontrolled schema/statistics/index/cache/role/session changes affect
estimates; observation is not ongoing freshness or exclusion of subsequent DDL.
There is no unconditional zero-cost/zero-lock/arbitrary-SQL side-effect promise.

## Acceptance evidence

C21 is central; C01/C06/C08/C11/C12/C13/C14/C15/C19/C23/C24 retain their existing
boundaries. Unit parser tests use synthetic sensitive canaries and independent
public JSON allowlists with detecting positive controls; these are not real DB
operator/packet observations. Dedicated live tests compare final SQL/args/metadata
and actual submission recorder events, typed native NULLs, independent context/
operation/session mismatch and deadline/cancel boundaries. Setup/observer SQL is
separate from production submission evidence; no test recorder is a wire packet
monitor. Private clock/barrier seams do not prove every real network cancellation.
Compiler tests include an actual compiling acquisition control and diagnostic-
checked rejection of serde, Clone, replacement, old-handle/permit/view/source
promotion and reuse. No public view/source JSON becomes execution authority.

Use only `scripts/test-dispatch.sh`'s new digest-pinned dedicated PostgreSQL16.14 /
MySQL8.4.8, explicit namespaces, limited roles, strict modes, FOUND_ROWS and InnoDB.
The script records actual version/settings and checks container/anonymous-volume
absence on success/failure. No production database is used. Exact commands, test
counts/failures/skips, source head, hosted CI, statement/counter evidence and cleanup
belong to this revision's PR/RelayWeft report. Windows hosted DB-free success is
separate from WSL real DB, Windows live/ACL and physical identity. All 24 full
`runtime_pending` cases remain. PR2 collector/budgets/N+1, Issue36/review changes,
external delivery and migration/authorization expansion remain outside PR1.

### Local finite regression

WSL Ubuntu 24.04.5 / Rust 1.94.0: dedicated full workspace/all-feature run passed
270 tests, 0 failed/ignored/filtered (263 retained + three parser/version tests and
four compiler-fail doctests). The compiler harness passed 391 diagnostic-checked
negatives +22 compiling controls (377+21 retained); generated consumer 76 negatives,
eight controls and six invocations remain unchanged. Default workspace passed114.
Full run verified PG16.14 and MySQL8.4.8 settings above and dedicated container/
anonymous-volume removal/absence. Both production EXPLAIN live suites passed.

C21 per dialect: positive auxiliary1/EXPLAIN1/business0/provider0/Tx0; PostgreSQL
six native NULL types and MySQL eleven each retain exact native arguments.
Result-budget, real missing-column driver refusal and actual return-time expiry
have auxiliary1/EXPLAIN1/business0. Entry context/operation/session/expiry mismatches
have auxiliary0/EXPLAIN0. Post-bind expiry/timeout/cancel have auxiliary1/EXPLAIN0;
barriers/one clock seam are explicitly test-only. A separate raw SELECT1 metadata
control is not a production submission. Independent observer checked two seed rows
and column name/type/nullability/order before/after. These are bounded real-driver
submission observations, not a full wire/server statement monitor. Unsupported
versions/nodes, malformed numeric/JSON/budgets and canary result fields use the
production parser with synthetic unit inputs, not arbitrary live operators.

Development failures were not counted as acceptance: an omitted recorder `send`
event expectation; oversized test future stack (fixed by boxing its added portion);
initial refusal of MySQL's finite impossible-WHERE result; and a test-only attempt
to use the disabled Tokio join macro (replaced with existing futures-util join).
Every failed disposable-DB trial also verified dedicated cleanup. Exact final head,
remaining feature/hosted checks and limitations belong to this work's report.

A sandboxed CLI matrix attempt hit EPERM on its required 127.0.0.1 listener
positive control. An approved listener/connect/accept diagnostic succeeded; the
affected regression and remaining checks use that approved local-fixture scope.
No DNS/firewall/global DB/host policy settings were changed.
