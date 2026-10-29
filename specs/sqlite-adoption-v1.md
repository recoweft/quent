# QT-RW-00 PR1: bounded SQLite adoption contracts v1

Refs #53. Design and future acceptance expectations only. **SQLite is not currently
implemented**: Core `Dialect` contains PostgreSQL/MySQL only, and the facade has no
SQLite feature, adapter or native decoder. No statement here authorizes execution.
This PR adds no production SQL, driver, dependency, runtime, migration or permission.
The current [adoption matrix](adoption-v1.md) remains authoritative for available APIs.

## Baselines and evidence boundaries

Quent baseline: `9e0b5787e766d47a3fa05a58e74ea67a0fca615a` (merged PR52).
Application investigation target: `a03091305bab18154dea89eb0aee6a35988d927e`.
Its commit object/tree was accessible read-only, but this PR deliberately uses only
the supplied anonymous requirements; application SQL/source/payload was not read or
copied. The full callsite inventory and byte compatibility evidence await private
QT-RW-00 PR2. Availability of a commit is not completion of that investigation.
Goquent reference actually read: `c51e084d0d530cae3eb22ad2b0686e47c6b79f0c`,
contracts v3 introduction/API table; documentary comparison only, no runtime or
wire compatibility evidence. Neither reference repository is modified.

Every candidate below requires a later finite implementation and executable oracle.
Unknown, Partial, unsupported facts or exhausted budgets refuse, with no old-SQL
fallback, automatic retry or public authority restoration. SQLite proposals must
feed the same private immutable IR, one finalizer, retained review record, common
policy/risk/final dispatch gate and finite native binding/decoding. A second SQL
renderer, arbitrary SQL/function/driver callback or Raw escape hatch is excluded.
Plan/compile/review remain DB/provider0. Connection acquisition, live observation,
EXPLAIN and migration remain separate explicit application boundaries. Core stays
zero-dependency; default facade/CLI and mcp-only stay driver/runtime-free. Existing
PG/MySQL gates, native counts, output allowlist and canary controls remain unchanged.
Rust1.94/edition2024/pins/lock/publish=false remain unchanged.

## Native values, keys and application scope

| Candidate domain | Required proof / positive expectation | Mandatory refusal / unresolved detail |
| --- | --- | --- |
| TEXT | Exact UTF-8 binding and full decode; preserve empty text and bytes of stored JSON text | No implicit numeric/bool/JSON coercion, normalization or lossy replacement |
| INTEGER | Exact signed i64 storage/decoder with checked narrower target ranges | REAL, overflow, text-to-integer and unsupported storage class refuse |
| SQL NULL | Explicit nullable typed NULL; distinguish missing input, empty text and zero | Missing required value or nonnullable NULL refuses; no default synthesis |
| Other storage | None selected in PR1 | BLOB/REAL/date/decimal/UUID and broad native extensions remain unresolved; ledger byte transport needs its own finite protocol design |
| TEXT key | Full current key components, NOT NULL, exact equality semantics, declared affinity and collation matching comparison/binding | A name, LIMIT, successful decode or string equality alone proves no key |
| Composite key | Every component bound in every possible TRUE branch; tuple equality/dedup uses proven database semantics | Prefix/range/IN/unknown OR branch cannot become Single |
| Nullable/partial unique | Only after a future explicit theorem covers nullability and partial predicate implication in every branch | No generic uniqueness inference; unknown implication/nullable uniqueness refuses Single |
| Collation/affinity | Independently current application registry plus explicit supported collation/affinity and actual SQLite oracle | NOCASE/custom collation, expression index, coercive comparison or stale facts stay Unknown until separately bounded |

The initial candidate is nonnullable TEXT/BINARY and INTEGER complete ordinary keys;
this is a proposal, not an expansion of today's signed-key proof. Single means at
most one target row, never existence, exact count or all indirect effects. SQLite
rowid is not stable business identity or default result order. WITHOUT ROWID,
INTEGER PRIMARY KEY aliases, tie ordering, nullable keys, index collations and
partial unique predicates require the PR2 inventory and later finite scope tests.
No hidden rowid insertion or schema rewrite is part of initial adoption.

| Scope kind | Trusted application responsibility | Refusal boundary |
| --- | --- | --- |
| project | Authenticate subject/project relationship and current resource mapping | Project labels from JSON are not tenant authority |
| connection | Bind exact connection identity, allowed operations, physical file and settings generation | Another connection/file or stale settings refuses |
| local-system | Supply explicit local-system capability and operation boundary | Never synthesize a tenant to pass the existing tenant gate |
| singleton | Prove target identity and bounded cardinality independently | Table name or convention alone proves nothing |

The application owns current schema/policy/settings/subject registry, physical file
identity and routing, filesystem/OS access controls and connection configuration.
Paths, constructors, manifests, Fresh, Low and type success authenticate none of
these. Symlink/replacement/attached-file routing, triggers, extensions, virtual tables
and concurrent schema changes require explicit supported evidence or refusal.
No new non-tenant authority mechanism is approved here: a later authority design
must consult before introducing an irreversible permission decision.

## Synchronous adapter and completion proposal

A future opt-in synchronous adapter may borrow an explicitly supplied SQLite
connection. It must not silently install an async runtime, open files during plan
or run application callbacks. Driver/package/version and thread/cancellation model
await the inventory and separate implementation review. Explicit read-only opening
must reject writes, creation and unapproved attachment; a read-only label alone is
insufficient. A deferred read transaction establishes a snapshot at the first read,
not at BEGIN; tests must pin that boundary across multiple connections. A bounded
write unit proposes BEGIN IMMEDIATE to expose writer contention before business
submission. No implicit upgrade/retry is assumed.

Application initialization must explicitly establish and independently observe WAL,
synchronous=FULL, foreign_keys=ON and finite busy_timeout for each applicable
connection. No PRAGMA mutation is performed by this PR or by offline review.
WAL is persistent file state; foreign_keys/busy_timeout are connection obligations.
Read-only/open/transaction constraints may prevent settings changes: incompatible
or unobserved settings refuse. Numeric timeout/performance budgets are not invented;
PR2 must measure and select them. Exceptions, schema migrations and backup/checkpoint
procedures retain current application ownership and need explicit inventory/approval.

| Event | Finite proposed outcome and required oracle |
| --- | --- |
| Read-only write / invalid settings | Refuse before business send; separately count open/setup/observation |
| BUSY/LOCKED at BEGIN | No business send; record begin attempt versus ACK, no autoretry |
| BUSY/LOCKED during statement | Preserve actual transaction state; rollback explicitly if owned; never infer no effects from the error alone |
| All operations and result checks succeed | Commit attempt; Committed only on confirmed commit ACK |
| Decode/count/constraint failure | Do not save success; rollback attempt, RolledBack only on rollback ACK |
| COMMIT error / missing completion evidence | Record whether transaction is still active if reliably observable; otherwise Unknown; no blanket rollback claim |
| ROLLBACK error, drop, interruption or crash | Best-effort cleanup is not ACK; consumed handle stays consumed; quarantine uncertain connection |
| Unknown reconciliation | Fresh explicit connection, current authority, same durable ID/namespace/full input and authoritative ledger; no replacement ID or unconditional resend |
| Borrowed completion | Caller-owned Uncommitted; library cannot promise final commit/rollback |

Fault tests must distinguish before-send, before-commit, durable-commit/missing-client
receipt and ACK controls, plus restart/independent file observers. SQLite local
errors differ from network reply loss; do not copy PG/MySQL fault claims wholesale.
Native SQLite changes/count, unchanged UPDATE, ignored constraint and UPSERT branch
counts need measured finite rules; never reinterpret them as PG/MySQL matched counts.
Full bounded rows/columns and exact per-operation expectations precede commit.

## Closed candidate expression and SQL register

“Candidate” means future positive subset conditional on complete facts; every other
shape refuses. No parser for arbitrary application SQL is proposed. Bounds must be
explicit in later APIs and verified against the callsite inventory.

| Family | Closed candidate subset | Mandatory refusal / inventory decision |
| --- | --- | --- |
| JOIN ON | Explicit INNER/LEFT equijoin of registered aliases and typed columns; scope proof on every source | Unknown relation, ambiguous column, arbitrary ON expression or scope omission |
| USING / aliases | Explicit resolved common columns and distinct registered aliases; declared projection | Implicit star/name resolution; merged USING projection semantics unresolved until oracle |
| LEFT JOIN | Right-side projection nullable after NULL extension; ON versus WHERE retained | Treating nullable right result as nonnull or moving filters across join |
| Multiplicity | Explicit one-to-one proof or bounded full result expectation | Parent-key Single does not bound joined row multiplicity; no first-row truncation |
| DISTINCT | Explicit finite projection, SQL NULL/collation equality and full bounded result | Dedup as a scope proof or undocumented ordering |
| EXISTS / correlation | Bounded registered inner source, explicit outer-column bindings and predicates, policy on both scopes | Unresolved correlation, arbitrary nested subquery or missing inner authority |
| COALESCE / CASE | Closed typed arms, explicit nullable result and SQL three-valued condition semantics | Mixed/coerced result types or arbitrary function/branch callback |
| Arithmetic | Checked finite INTEGER operations, explicit NULL/overflow/division semantics | REAL promotion or overflow silently accepted; exact operator list awaits PR2 |
| LIKE | Typed TEXT, explicit escape/case/collation setting, bounded input | Implicit case mode, custom function or assumed index/key proof |
| IN | Finite typed values, explicit NULL truth table and empty-list false | NULL-as-false rewrite or coercion; IN alone is not complete key equality |
| Order | Explicit fields/directions, NULL placement, collation and proven tie key for stable pages | Implicit rowid/order or nonunique pagination assumption |
| SQLite JSON | Fixed registered path, closed scalar projection with presence/type discrimination | Dynamic path, arbitrary JSON function or missing/type coercion |
| UPSERT | Explicit complete conflict target, supported unique facts and separate insert/update policy proof | Unnamed target inference, excluded scope/readonly assignment or unproved update branch |
| excluded | Typed reference to proposed row only within supported UPSERT branch | Authority from proposed scope values or arbitrary pseudo-table access |
| IGNORE | Individually classified constraint outcomes, exact count/result expectations | IGNORE is not general DO NOTHING: NOT NULL/CHECK/unique behavior cannot be conflated; FK/error cases separate |
| INSERT SELECT | Finite source projection into explicit destination columns with both policies, types and cardinality checked | Source scope omission, unbounded result or partial branch acceptance |
| Mandatory unresolved | REPLACE, generic GROUP BY, CTE, window, broad native types and DDL | No advance implementation; REPLACE delete/insert effects require separate decision |

JSON requires separate cases for absent path, JSON null, SQL NULL input, text,
integer, boolean, array/object, malformed document and wrong type. json_extract may
collapse missing and JSON null to SQL NULL: presence/type must be retained when the
application distinguishes them. Text JSON storage/response bytes must not be
reserialized. SQLite JSON version/operator/error behavior and JSONB are not assumed;
unsupported/error cases refuse before a business write or roll back with ACK.

## Typed finite multi-operation Tx and existing operation ledger

Current APIs offer single-operation atomic execution and a fixed business/audit/
outbox bundle, not arbitrary multi-op Tx. QT-RW-05 must design bounded immutable
ordered typed operations, exact per-op result schemas/counts, consumed ownership,
all-operation preflight and per-dispatch revalidation through the existing gate.
No arbitrary closure, driver callback, Raw or exposed executor is permitted.
Bounds/branching and response protocol are unresolved until the full inventory.

The existing application operation ledger is the integration target. Do not require
a second Quent ledger, overwrite old bytes or pretend QCK1/QCC1/QCR1 already encode
SQLite/application responses. QT-RW-00 PR2 and QT-RW-05 PR3 must document the exact
full input, namespace, durable ID, current authority, existing response bytes and
version/retention/integrity rules. Domain changes plus saved response must share
one SQLite file/connection/owned transaction; cross-file atomicity is not assumed.

| Scenario | Future acceptance requirement |
| --- | --- |
| New ID | Persist complete original input before first send; domain + response commit together |
| Same ID / same full input | Revalidate current authority; exact saved response bytes replay, zero business resubmissions |
| Same ID / different input | Conflict, no overwrite, no business send; comparison is full canonical input, not hash alone |
| Corrupt/truncated/unknown-version/pending row | Refuse, never interpret as absent or repair automatically |
| Concurrent same ID | One durable winner; loser explicitly reconciles; missing lookup does not prove winner absent |
| Crash/restart / Unknown | Same ID/full input/current authority, fresh explicit lookup and independent observer; no automatic new ID |
| Failed middle op / response save | Whole owned unit rolls back only with ACK; observer checks domain and ledger together |
| External I/O | Save, adopt, deliver and execute are distinct transitions; database Tx cannot make external I/O atomic or exactly-once |

## Gate A and Gate B

Gate A is application adoption, after all target DML callsites have an exhaustive
classification and corresponding positive/refusal evidence in private PR2's full
ledger. Coverage denominators and exceptions cannot be inferred from anonymous
fixtures. Record old/new result semantics, old data/IDs/full input/stored response
and JSON bytes; retain existing schema, migration and backup initially. For every
callsite either a finite supported path or a blocking unsupported decision is
required, without hidden fallback. Exceptions (PRAGMA, migration, backup, maintenance)
need owners and separate boundaries rather than a blanket DML success label.

Gate A additionally requires actual SQLite files, multiple connections, locking,
snapshot boundaries, fault/recovery/restart and independent absence/ACK evidence;
Windows native and WSL separately; retained PG/MySQL regressions; measured baseline
latency/throughput/file-size/lock-wait budgets and selected thresholds in PR2.
No unmeasured budget is supplied here. Hosted Windows DB-free CI is not local
Windows SQLite/file/ACL evidence; earlier dependency/policy stops stay unverified
and must not be bypassed. Gate A remains unmet by this PR.

Gate B requires Gate A plus a finite published support/refusal matrix, consumer
build/package checks, dependency/license/name availability review and exact original
case differences. It does not clear all24 runtime_pending or authorize release,
version/license changes, publish=true, tags or commercial-safety declarations.
Public candidate status stays on hold pending those measurements and decisions.

## Independent anonymous fixture and original-case delta

[sqlite-adoption-v1 register](../fixtures/sqlite-adoption-v1/cases.json) has its own
kind/version and synthetic inputs. It is documentation data, never trusted registry,
executable SQL, a permit or a production-loader input. Each positive and refusal is
an expected future observation; every runtime status is pending. Structural JSON
validation does not execute SQLite, validate scope proofs or measure runtime counts.
No new loader/schema test is needed for a data-only register with no consumer.

| Original cases (unchanged) | SQLite adoption delta, no full-case completion |
| --- | --- |
| QT-C01–07 | Native TEXT/INTEGER/NULL, scope/key/collation and finite expressions/joins |
| QT-C08–15 | Explicit non-tenant authority design, connection/current registry and consumed handles; output redaction unchanged |
| QT-C16–18 | Synchronous snapshot/write Tx, native counts, existing-ledger bytes and crash reconciliation |
| QT-C19–22 | Settings/file observations, inventory coverage, measured budgets; migration/backup remain application-owned |
| QT-C23–24 | No SQLite MCP execution or wire-vocabulary expansion; existing refusals retained |

Original [contracts-v1](../fixtures/contracts-v1.json), loader vocabulary and
[all-case register](integration-acceptance-v1.md) retain their expectations and all
24 runtime_pending. SQLite fixtures are neither added to that loader nor counted as
passing original cases. Future tests must construct independent expected outcomes,
not read a fixture verdict and return it as evidence.

## Dependency and repository responsibility map

The whole plan has 10 management items, 13 repository Issues and 23 planned PRs.
Actual future PR numbers/URLs are not yet assigned here. Each same-Issue PR follows
its predecessor merged into the real base; contracts alone do not satisfy runtime gates.

| Item | Quent responsibility | Private application responsibility / dependency |
| --- | --- | --- |
| QT-RW-00 | [Issue53](https://github.com/recoweft/quent/issues/53) PR1: this contract/anonymous fixture | PR2: full callsite ledger, bytes, budgets; whole00 waits for both repositories |
| QT-RW-01 | [Issue54](https://github.com/recoweft/quent/issues/54): dialect/native/TEXT-composite proof | Uses00 inventory; no inferred app migration |
| QT-RW-02 | [Issue55](https://github.com/recoweft/quent/issues/55): synchronous adapter/snapshot/Tx | Depends on01 domains and00 settings/OS inventory |
| QT-RW-03 | [Issue56](https://github.com/recoweft/quent/issues/56): finite expressions/JOIN/EXISTS/JSON | Uses01/02 and00 query classification |
| QT-RW-04 | [Issue57](https://github.com/recoweft/quent/issues/57): UPSERT/conditional writes/scope | Uses01–03 and00 conflict/effects inventory |
| QT-RW-05 | [Issue58](https://github.com/recoweft/quent/issues/58): finite multi-op Tx/ledger contracts | PR3: existing ledger integration; depends on00 and relevant02–04 operations |
| QT-RW-06 | No Quent Issue | PR1–2: coexistence with existing migration/backup; upgrade, backup/restore, read-only/newer-version refusal and failure recovery; depends on02/05 |
| QT-RW-07 | No Quent Issue | PR1–3: staged adoption from read pilot → project/work mutation/replay → remaining shared/answer/execution DML; depends on03/05/06; no implicit fallback |
| QT-RW-08 | [Issue59](https://github.com/recoweft/quent/issues/59) PR1: anonymous common acceptance/regressions | PR2: app E2E/OS/performance after06/07; public fixtures cannot substitute |
| QT-RW-09 | [Issue60](https://github.com/recoweft/quent/issues/60): adoption/public-candidate decision | Requires08 evidence, Gate A/B and unresolved decisions |

This is a dependency outline, not a new allocation of the 23 PR scopes. Follow-up
API/PR splits may be updated from the full inventory through the relevant Issues.
Only00 PR1 is authorized here. Private source/SQL/full payload/credentials remain
outside public Quent, PR/CI/MCP artifacts. Public output remains fixed allowlist-only;
synthetic fixture names/values below originate here, not from application source.
