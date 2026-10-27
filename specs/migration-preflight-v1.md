# QT-AI-11 PR2: phases and finite migration observations

This extends the [immutable PR1 plan](migration-plan-v1.md), without expanding its
SQL subset. It adds no apply, executor, authorization provider, permit, CLI/MCP
execution, Raw query or callback. Issue #12 and every full runtime_pending marker
remain open/pending. PR3 owns human-controlled application.

## Pure phases and exact record binding

`MigrationPlan::phases()` projects three private immutable records in order:

| Phase | SqlPlanned nullable-append plan | NoChange | Refused/unknown plan |
| --- | --- | --- | --- |
| Expand | NullableAppend; contiguous steps `[0, final_sql.len())` | NoWork | Unsupported, zero steps |
| Backfill | NoWork; empty range at end | NoWork | Unsupported |
| Contract | NoWork; empty range at end | NoWork | Unsupported |

The sole work capability is PR1's constraint-free nullable signed-integer tail
addition. NoWork describes this particular plan; it does not implement arbitrary
backfill writes or destructive contract. Unsupported is never converted to
NoWork. Each private record retains its exact range and three ordered finite
conditions: BeforeColumns, NoIndexesOrConstraints, FiniteTableSemantics. The
original exact before/after snapshots, authority, target, ordered differences and
final SQL remain private. `same_record` compares full retained content, including
phase records; no hash, SQL string alone, legacy manifest ID or public summary
decides identity. Public phase counts cannot restore a plan or evidence.
Planning, compiling, review, phase projection, request construction and comparison
have no DB/provider I/O. Core remains dependency-free. Existing plan-view JSON,
CRUD summaries, manifest wire/canonicalization/fingerprints remain unchanged.

## Explicit application-only API

With `manifest` + driver (`postgres` or `mysql`) + `runtime-tokio`, use the existing
`manifest::live::{PostgresReader,MysqlReader}` and the new `live::preflight` types:

1. Supply an actual private MigrationPlan, Expand, explicit Scope, Visibility
   (exact role and nonzero application visibility generation), and DdlExclusion.
2. Construct `PreflightRequest::new`; Refused, NoWork, uncontrolled concurrent DDL,
   non-application authority, or a scope differing from the exact plan refuse.
3. Explicitly connect the dedicated reader using existing checked options and
   DatabaseContext. No request creation connects or fetches a catalog.
4. Explicitly `observe_migration(current, request, budget, ttl).await`.
5. Independently `compare_migration(current, current_request, evidence)` before
   relying on the finite observation. This samples the current process clock and
   compares exact independent current inputs. There is no current-clock JSON API.

Readers privately own their connection, context and allocation identity. They
expose no driver, session role change, arbitrary SQL, external transaction,
reconnect/retry or callback. Source public claims/JSON cannot construct evidence.
The caller authenticates application context, role, routing and the physical
server; address/database labels and matching CURRENT_USER are not authentication.
Same labels on a different reader cannot recheck the original evidence. Keep the
reader session, role, visibility generation and explicit scope fixed. Applications
must supply genuinely current snapshots; replaying obsolete application context
is not independently detected as a new fact.

## Statements, queries and finite coverage

The consultation answer for question `8bcc7b2e-2708-4b2f-b9c9-782c7683f802`
clarifies the boundary as **finite SELECT plus fixed read-only observation Tx
control**. Every observation uses at most one fixed BEGIN/START and at most one
explicit ROLLBACK, under the original total timeout. Success requires both ACKs.
No COMMIT, migration apply BEGIN, DDL, DML, business write, arbitrary function,
user-table lock extension, provider call or authorization issuance is introduced.

| Area | PostgreSQL 16.14 | MySQL 8.4.8 |
| --- | --- | --- |
| Tx | BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY | START TRANSACTION READ ONLY |
| Identity SELECT (once) | current catalog/schema and exact version | catalog/version, GIPK visibility and case settings |
| Existing header SELECT/table | base heap/privileges and finite trigger/RLS/inheritance/rewrite/index restrictions | base InnoDB, schema-wide direct SELECT+TRIGGER grants, triggers/constraints/partitions |
| New extension SELECT/table | exact namespace/table/CURRENT_USER, count of **all** local indexes and local constraints, dropped-attribute flag | exact namespace/table/CURRENT_USER, all local statistics rows and table-constraint counts |
| Existing columns SELECT/table | ordered native signed integer/null/default plus generated/identity/collation flags | ordered signed integer/null/default plus EXTRA/generation/collation/type flags |
| Existing keys SELECT/table | unique keys, retaining old reader behavior | index rows, retaining old reader behavior |
| Cleanup | explicit ROLLBACK with ACK | explicit ROLLBACK with ACK |

New extension queries are static SELECT constants in the existing closed adapters;
only namespace/table bind arguments are accepted. They execute inside the same Tx,
quiet-driver wrapper, shared Meter and total timeout as the old SELECTs. No second
free SQL route is created. Old observe/compare remains unchanged and never receives
the extension option. Its Observation cannot be converted into preflight evidence.

For N tables: `1 + 4*N` catalog SELECTs, at most 65 for 16 tables; plus at most two
fixed Tx controls, separately counted. Handshake/protocol traffic is not included
in that SELECT total. Shared ceilings: <=8192 rows, <=1MiB encoded result bytes,
<=4096 bytes per field, <=65536 bytes per encoded row, <=16 fields, <=256 columns,
<=16 keys, <=256 branches, timeout `(0,60s]`. Scope is 1..16 unique tables in one
explicit lowercase finite namespace. Unsupported multiple namespaces or native
before-column types outside signed integers refuse. New extension expects exactly
one six-field record/table, exact namespace/table/role and zero counts/no dropped
attributes; missing/duplicates/multiple/reordered/unknown/badly typed facts cannot
be evidence. Column field count, nullability token and ordinal progression are
additionally checked. Fixed row sentinels (2/257) and the shared Meter refuse
oversize; no successful truncation. This bounds retained results and total waiting,
not arbitrary server CPU cost before cancellation.

The plan's target/authority/before/after/ordered steps remain **application
declarations**. Live matching covers the selected before-column order, signed
native types, nullability, supported literal/default knowledge, absence of local
indexes/FK/CHECK/other catalog constraints, and the existing finite table-semantics
flags, under checked role visibility. New expected-column projection is private;
it is not a public ManifestSource-to-MigrationSchema conversion.

No index/FK/CHECK definitions are implemented: nonempty sets refuse finite success.
No all-effects catalog completeness is claimed. Incoming references outside scope,
server extensions, physical routing, complete permission models and effects outside
the table header remain unverified. The selected finite facts do not prove absence
of every possible unrepresented effect. Full source TableEffects::Ordinary remains
an application assertion. After schema and future added columns are bound to the
plan, not observed as already applied.

PG normalized missing attrdef and MySQL nullable COLUMN_DEFAULT=NULL cannot prove
Absent versus explicit NULL syntax. Such observations remain Incomplete, even if
the application claims Absent. A positive finite example uses literal defaults on
all existing columns; new nullable absent-default columns exist only in the bound
future plan. MySQL's nonnullable no-default case retains the old reader's finite
knowledge; it does not prove original source spelling. Opaque expressions,
unknown/unsupported metadata, missing grants and partial visibility never match.
Offline NoChange/SqlPlanned/source equality is not current-DB validation or apply
success, and is never a constructor of live evidence.

## Evidence, clock and limits

`PreflightObservation` is private immutable, non-Clone/non-Copy, no Display/serde,
and borrows the original immutable plan. It retains exact request (plan, phase,
conditions/range, scope/visibility/exclusion), full DatabaseContext, reader identity,
actual finite catalog data and extension records, start/completion and expiry.
TTL starts before reads: `(0,300s]`, checked addition, exclusive expiration
`now >= expiry`. Monotonic Instant is authoritative; checked wall timestamps are
private descriptive Unix milliseconds. Backward/future samples, wall time before
Unix epoch/overflow, backward wall time, late cleanup/comparison and expiration
refuse. Wall jumps do not extend TTL. No persistence, clock injection, imported
wall-time Fresh, cross-process cache or public evidence restoration exists.

Exact current plan/target/context/phase/conditions/visibility/role/exclusion changes
fail comparison; mismatch does not make new observations. A valid borrowed
observation may be compared repeatedly within TTL: evidence is observational, not
an execution handle. It cannot enter CRUD validation or PR3 application. No new
linearity/exactly-once execution claim follows.

A complete successful finite read plus actual ROLLBACK ACK is required before
minting evidence. Drift, unknown, missing, shape/budget error, timeout/cancel,
connection/read/cleanup failure return a closed error and no success evidence.
After I/O begins the reader is unavailable until the full success boundary; failure
or dropped future quarantines it. It cannot revalidate old evidence after such a
failure. Unpolled futures do nothing. A Tx Drop/socket close/quarantine is not a
ROLLBACK ACK. Error cleanup is attempted inside the remaining original timeout;
no fresh timeout/retry is added. All packet-level failure boundaries are not
claimed as measured; see the evidence register.

The output-only `PreflightView` has exact kind/version/state/coverage/phase/steps/
tables/conditions/full_effects/authority fields. Success is **ConditionsObserved**,
with overall **Partial** coverage and **Unverified** full_effects. There is no
Ready state, complete migration preflight claim, authorization, DDL permission or
apply handle. Unknown facts are not upgraded by using an alternative success name:
only the three enumerated finite conditions have succeeded; every omitted domain
remains unverified and cannot satisfy a future full-schema apply gate. Other states
are Mismatch, Stale, Unverified. Errors are closed fixed variants with no external
cause. Debug/Display/JSON carry no names, SQL, values, role, generation, target,
arbitrary errors or opaque metadata. No new external input wire, CLI/MCP route,
tracing emitter, SARIF or local sensitive artifact is added. Existing protected
new-file export and Windows UnsupportedOs remain unchanged.

PG observes the measured catalog SELECTs' repeatable-read MVCC snapshot, without
DDL exclusion locks or a post-observation guarantee. MySQL information_schema does
not become a whole-catalog MVCC snapshot by using a read-only Tx. For both dialects
this API conservatively requires `DdlExclusion::ApplicationDeclared(nonzero_epoch)`;
its exact generation is bound/rechecked, but the external exclusion mechanism is
not verified by Quent. Uncontrolled DDL refuses before I/O. The caller must maintain
that condition during capture and decide when a new observation is needed. A later
external DDL is detected by a new explicit read, not by an I/O-free recheck. Nothing
promises zero locks, data-loss safety for other migrations, lasting freshness,
authenticated server identity, successful DDL application or universal rollback.
PG transactional DDL and MySQL ALTER implicit commit remain PR1/PR3 limitations;
observation ROLLBACK is not a migration/business transaction rollback.

Finite tests and actual measurements: [PR2 evidence](migration-preflight-v1-evidence.md).

Current QT-AI-11 PR3: [explicit human-controlled finite application](migration-apply-v1.md), [finite evidence](migration-apply-v1-evidence.md). Historical PR1/PR2 absence statements retain their original scope; no full runtime_pending marker is cleared.
