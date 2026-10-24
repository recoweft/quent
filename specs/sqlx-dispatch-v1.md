Current PR3 additions: [finite owned atomic results, version/RETURNING IR and borrowed Tx boundaries](atomic-results-v1.md). Earlier PR-specific absence statements below describe their historical baseline.

Current PR2 behavior: [consumed validation, current input/session binding, expiry
and cancellation stages](owned-validation-v1.md). The PR1 baseline below remains
historical where it defers delayed handles; its finite gate/native matrix is preserved.

# QT-AI-03 planned PR1: immediate SQLx dispatch

Only `PostgresSession` (`postgres,runtime-tokio`) and `MysqlSession`
(`mysql,runtime-tokio`) add explicit connection and immediate execution I/O.
`tls-rustls` remains optional. Default facade and Core remain driver/runtime free.
Plan/finalize/inspect/policy preparation still perform zero DB/provider calls.
CLI/MCP receive no execution capability. Typed models/codegen are QT-AI-07.

## One private pipeline

`session.execute(&current, owned_operation)` uses the one shared private dispatcher
macro and gate in `execution.rs`: exact snapshot/dialect/subject check → existing
finite policy preparation → sole Core finalizer → inspection of that same retained
private Plan record → finite native encoding → exact context check → one adapter
send. All five IR families enter this gate, including dynamic tables and COUNT.
No public function accepts SQL, raw Plan, SensitiveSource, PolicyPreparation,
public summary, callback, binder, Any or boolean permission as execution input.
The Core/facade boundary uses only private-constructed immutable preparation and
its finite borrowed source; result types are read from that same record's metadata.
No SQL parsing, second renderer, append-after-review or family-specific gate exists.
A caller cannot manufacture a replacement record, pass one into execution, or
recover the session's raw connection. Compile-fail examples cover these boundaries.

| Gate condition | Before-send behavior |
| --- | --- |
| Complete + Low/Medium + complete supported schema/policy/subject/target context | Candidate for the finite adapter; application authentication/business authorization are still required |
| Missing/inconsistent context/coverage, unsupported table semantics/identity/native type | Refuse |
| Partial/Unknown analysis or Unknown scope | Refuse even with a structural Single bound |
| Blocked, tenant mismatch/assignment, PII/protected/RequiredFilter violation | Refuse; no reason or suppression override |
| High/Destructive | Refuse: external authorization/provider permit is not implemented |
| INSERT | Always refused today: InsertUnverified/High; no demonstration risk downgrade |
| Complete SELECT/COUNT Low/Medium | Execute; Medium/unlimited read still has bounded result consumption |
| Complete filtered UPDATE/physical DELETE Zero/Single Low | Execute; Single means at most one conditional target, never existence/exactly-one/atomicity |
| Broad/finite-many writes, unsupported bool/text/nullable predicate abstraction | Refuse according to retained High/Destructive/non-Complete inspection |
| batch/upsert/RETURNING/join/Raw/CTE/subquery/unsupported SQL type | No supported IR entry; refuse at construction/preparation, never lower just the first candidate |

SoftDelete's bool guard currently yields Partial scope analysis and therefore
refuses execution. RequiredFilter's existing signed equality/all-OR-branches rule
and PII refusal remain unchanged. Empty policy sets do not authenticate/authorize.
Fresh manifests, reason/PII reason, native type success and Single do not waive gates.

## Session and current-context contract

`connect(typed_sqlx_options, owned_context)` is explicit I/O. It owns one connection
and the entire immutable application-supplied context; there is no pool/executor
replacement API. SQLx statement AND slow-query logs are disabled before connecting.
The connect await completes before the immediate execute API is used. Execute has
no application await between preparation, native binding, exact full-content context
comparison and query submission. `&mut session`, borrowed immutable current context,
and the owned immutable operation prevent application-side replacement across the
send await. No callback receives an executor. A different subject/tenant/schema/
settings/policy generation/content/target/routing key/dialect refuses before send.
Changing context requires a new session; no partially bound reusable handle exists.

This is application assertion correspondence, not physical database authentication.
The application must supply authentic subject/tenant, current authoritative schema/
policy/settings and correct DSN-to-routing association. Supplying the same obsolete
snapshot as both bound/current, inaccurate ordinary-table assumptions, external DDL,
server replacement, triggers/RLS or wrong physical routing cannot be authenticated
by equality of nominal keys. The library performs no implicit catalog/provider I/O.
PR2 owns delayed consumable handles, expiry and current-input revalidation after
provider awaits. This PR exposes neither ValidatedOperation nor delayed permits.

## Native adapter and result matrix

| Core native type (including typed NULL) | PostgreSQL SQLx 0.9.0 | MySQL SQLx 0.9.0 |
| --- | --- | --- |
| I8 | Unsupported (SQLx i8 is PG internal char, deliberately excluded) | i8 / signed TINYINT |
| I16/I32/I64 | i16/i32/i64 / INT2/INT4/INT8 | i16/i32/i64 / signed SMALLINT/INT/BIGINT |
| U8/U16/U32/U64 | Unsupported | exact u8/u16/u32/u64 / unsigned corresponding width |
| Bool | native bool / BOOL | native bind bool; decode signed BOOLEAN/TINYINT(1) only, exact 0/1; 2 refuses |
| Text | native String, TEXT/VARCHAR results | native String, finite TEXT/VARCHAR wire forms with nonbinary character collation |
| Null(type) | Option::<native>::None for supported types | same |
| Missing / decimal / float / bytes / UUID / date/time / opaque | Unsupported; no coercion/Display fallback | same |

INSERT gate refusal and this adapter matrix are separate facts. Native binding is
actually exercised through permitted UPDATEs; INSERT is never sent by the dispatcher.
Signed extrema, unsigned maxima/zero, false/true/empty text and each supported typed
NULL are tested independently of raw setup inserts. Missing remains a refusal.

Pinned SQLx 0.9.0 official crate sources were read from Cargo's checksum-verified
registry: `sqlx-core/src/{database,query,sql_str,connection}.rs`, both drivers'
`types/{int,uint,bool,str}.rs` where applicable, MySQL `type_info.rs`,
`protocol/text/column.rs`, `connection/stream.rs`, and driver argument implementations.
0.9 has non-GAT `Database::Arguments` and `query(impl SqlSafeStr)`.
`AssertSqlSafe(record.sql())` is the documented safe marker for audited dynamic SQL,
not an unsafe conversion: only the sole validated/quoted Core renderer supplies it.
`try_bind` checks encoding before send. No transmute, leak or unsafe is used.

MySQL SQLx integer compatibility alone ignores native width; this adapter additionally
compares exact type-info width/sign before decoding. TEXT under utf8mb4_bin can have
SQLx name BLOB/VARBINARY: only those finite text wire forms WITH `String::compatible`
(nonbinary character collation) are accepted; binary bytes do not become Text.
COUNT uses final Count metadata and exactly one nonnegative native 64-bit result:
PG I64; MySQL I64 or U64 according to actual signedness. Tested MySQL 8.4.8 returns I64.
Projection uses exact final order/type/nullability; unexpected count/type/NULL refuses.

Results/errors/session Debug are redacted; errors have fixed enums/Display and no
SQLx source chain. Only explicit `sensitive_rows`/`sensitive_count` access yields data.
No SQLx raw row/error/query Debug is output. Returned rows are streamed, capped at
256 rows, 4096 bytes/text and 1 MiB total semantic payload. Over-limit returns an
error, never a truncated success. This bounds consumption/retained values, not SQLx
protocol buffers, server computation or one incoming packet allocation. Result
metadata is bounded by the existing Core budgets. No implicit SQL LIMIT rewrite.

Every session is marked unavailable before send. A read/encode/decode/limit error
or canceled polled send is not retried; post-send errors leave the session unusable.
A dropped unpolled immediate future sends nothing. Write driver errors return fixed
`OutcomeUnknown`, never confirmed failure/rollback. Canceling a future cannot itself
return a classification. Ordinary CRUD is not retried or assigned replacement IDs.
PostgreSQL matched observations and SQLx MySQL FOUND_ROWS observations are named
separately. MySQL's pinned driver enables FOUND_ROWS; unchanged UPDATE returned 1
in both tested dialects. No upsert/changed-row, exactly-one, commit or atomicity
promise follows. PR3 owns transactions, expectations, version and precommit decoding;
QT-AI-09 owns provider permits. C09/C13/C16 are not completed here.

## Reproducible live checks and evidence

Run `bash scripts/test-dispatch.sh` with Docker available. It creates NEW loopback
servers without persistent named volumes; no DATABASE_URL or existing service is
used. Digest pins: PostgreSQL
`sha256:16bc17c64a573ef34162af9298258d1aec548232985b33ed7b1eac33ba35c229`
and MySQL
`sha256:2952e3be7807f06fc18de50b3ea1a632d5c70d63482ff7d7376fe3aa8999babf`.
The script creates `quent_dispatch_test` DB/schema and `quent_test` namespace-only
role, waits for readiness with a finite bound, records version/settings, then runs
both `live_` unit tests. Table cleanup runs on assertion panic; EXIT removes both
containers AND anonymous volumes on test/setup failure as well as success. Cleanup
failure fails the command. Trust/empty-password test authentication is restricted
to these disposable loopback containers; it is not deployment configuration.

Observed WSL Ubuntu 24.04.5 / Rust 1.94.0:

- PG 16.14, Alpine, UTC, read committed, UTF8, en_US.utf8 catalog collation.
- MySQL 8.4.8, +00:00, REPEATABLE-READ, utf8mb4/utf8mb4_bin,
  ONLY_FULL_GROUP_BY, STRICT_TRANS_TABLES, NO_ZERO_IN_DATE, NO_ZERO_DATE,
  ERROR_FOR_DIVISION_BY_ZERO, NO_ENGINE_SUBSTITUTION; FOUND_ROWS enabled by SQLx.
- Features: both drivers/runtime-tokio/tls-rustls; no actual TLS handshake is claimed.
- `bash scripts/test-dispatch.sh cargo test --workspace --all-features --locked --offline`
  includes live tests, regressions and API compile-fail. Exact head is recorded in
  the PR/RelayWeft execution report rather than a self-referential source commit.

The finite cfg(test) recorder is physically at the production dispatcher after
preflight/native encoding and before SQLx execute/fetch. It stores exact sealed SQL
and ordered native args, with no mocked executor. Initial SELECT has independent
literal SQL/arg/result expectations for each dialect; permitted UPDATE checks native
ordered args, independent observer reads and same-value row counts. Negative gate
operations record zero additional calls. Dropped-table read/write errors record one
call with no retry and fixed ReadFailed/OutcomeUnknown. These are query-submission
observations, not wire packet, server statement-count or Tx events. Setup/observer
SQL is deliberately separate and never counted as business dispatcher evidence.

C01–08/C10–12/C15 have limited PR1 evidence: C02 original unfiltered blocks;
C03–05 broad/unknown rejection; C06 full-OR guard and tenant isolation; C07 all native
values/typed NULL/NOT UNKNOWN/empty IN; C08 no subject/schema/permit nonpromotion;
C10 tenant/protected policy refusal; C11 independent tenant sessions and distinct
policy snapshot rejection; C12 subject/dialect/tenant/generation/context mismatch;
C15 redacted result/error/source and real API compile-fail. Decode mismatch, invalid
MySQL bool, oversized text and row count are failures. Existing source fixture
bridges/oracles continue to test Core; scripted recorder-only tests are not counted
as production evidence. Full fixture `runtime_pending` stays on all 24 cases:
Tx inheritance, delayed handles, JSON/public views, provider/manifest, batch, atomicity
and broader responsibilities are unimplemented/unverified.

New production dependency: exact `futures-util =0.3.34`, default features off + std,
optional through runtime-tokio, only for bounded streaming TryStreamExt. It was already
pinned transitively; Cargo.lock only attaches it to quent. Core gains no dependencies.
SQLx =0.9.0, Tokio =1.54.1, Rust1.94.0/edition2024/publish=false are unchanged.
`check-features.sh` checks all 16 feature subsets. Required live-databases CI job
runs both DBs and fails on missing setup/config; DB-free all-feature CI explicitly
uses `--skip live_`, never counts those exclusions as DB acceptance. Repository
branch protection is separately administered; adding a CI job does not change it.

## QT-AI-04 PR1 output boundary

[Public views v1](public-views-v1.md) adds output-only finite projections of the
current retained record/inspection/preparation and fixed errors/result states.
Historical statements above deferring public views describe their named baseline.
No public JSON or copied display classification is execution authority. Sensitive
getters and the complete private gate remain unchanged; CLI/tracing/export remain
with PR2/owning issues. All 24 full runtime_pending markers remain.
