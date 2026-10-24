# QT-AI-05 PR2: explicit finite catalog verification

This extends [source v1](schema-manifest-v1.md); offline source canonicalization,
independent fingerprints and the Strict/context/owned/atomic execution boundaries
are unchanged. `quent/manifest` alone is still DB/runtime-free. Live APIs additionally
require runtime-tokio and the selected driver. Core has no new dependency. None of
these APIs converts source or observation into ScopeFacts, SchemaSnapshot,
ApplicationContext, validated/atomic handle, authorization or a permit. No Plan,
compile, inspection or offline comparison initiates a connection/provider call.

## Explicit API and target contract

`manifest::live::{PostgresReader,MysqlReader}::connect(options, context, timeout)`
explicitly opens a **dedicated** privately owned connection. It is independent of
business sessions, has no raw executor/driver getter/replacement/callback and never
runs business CRUD, DDL, DML, EXPLAIN, provider, migration or user-table SELECTs.
SQLx's fixed connector initialization belongs to connection setup, not catalog
measurement or business CRUD. Statement and slow logging are disabled; the existing
per-poll AND drop NoSubscriber wrapper is shared, without a global logger change.
Dropping a dedicated reader suppresses driver drop diagnostics as well.

This initial live subset supports explicit TCP endpoints only: the source and
application context address must equal `options.get_host():options.get_port()`
(exact spelling, including an unbracketed IPv6 host followed by colon/port). Socket
options refuse. Database identity is an application identity; catalog must exactly
match the explicitly configured SQLx database. No source field supplies connection
options. Routing labels accepted by the offline parser are insufficient for this
live subset. Host/address equality does **not** authenticate a physical machine,
DNS resolution, server replacement, certificate, role or application identity.
The application owns authentic options/context and physical target correspondence.
SQLx options construction outside the CLI, including pgpass/environment behavior,
is the caller's responsibility. No DSN/credential is retained in public output.

`Scope::new(namespace, tables)` is an explicit nonempty set: one namespace, 1..16
unique tables, 1..63 ASCII lowercase/digit/underscore names. Tables sort by exact
name. Scope is never inferred from a manifest. All source tables must be exactly
this set of two-component paths; independent schema target must match all four
context target fields. MySQL namespace must equal catalog. Before measurement the
reader checks budget, scope/source target and full current/bound context. In the
transaction the first fixed SELECT measures current database and current schema
(PG) / current database twice (MySQL's schema/catalog synonym); both must match the
explicit expectation. PG therefore requires the caller's current schema to equal
the requested namespace. No unqualified table/search-path resolution is performed.
Missing/invisible metadata gives Unknown, never verified absence.

## Catalogs, finite support and completeness

Each observation uses one identity SELECT plus three SELECTs per table (header,
columns, ordered index components), all fixed text with bound namespace/table
values. PostgreSQL uses pg_catalog class/namespace/attribute/type/attrdef/index,
constraint/trigger/inheritance/rewrite/operator-class/access-method metadata.
pg_get_expr is called only for <=4096-byte stored node trees whose every CONST
uses built-in bool/int2/int4/int8 OIDs (16/21/23/20); other constant types receive a
fixed unsupported marker without deparsing, so custom type output functions cannot
run merely to render an unsupported constant. Deparsed nonliteral syntax is still
Unsupported; only the closed checked integer/bool literal grammar can be Complete.
No expression or default is evaluated.
MySQL uses information_schema TABLES/COLUMNS/STATISTICS/TRIGGERS/
TABLE_CONSTRAINTS/PARTITIONS/SCHEMA_PRIVILEGES. No source SQL is executed and no
identifier is interpolated into catalog SQL.

| Observed feature | Finite behavior |
| --- | --- |
| PG ordinary heap table, built-in i16/i32/i64/bool; MySQL ordinary InnoDB i8/i16/i32/i64 and unsigned equivalents | Exact native type and nullability, no tinyint-to-bool inference |
| Column order | PG attnum / MySQL ORDINAL_POSITION order retained (including physical gaps); order changes differ |
| Primary / unique key | Ordered components retained; keys sorted by kind then complete component sequence, independent of index name / OID |
| Key proof subset | Only nonnullable signed integer components are Complete, consistent with PR1; nullable, unsigned, bool/text keys unsupported |
| No default | PG missing attrdef is Unknown (explicit NULL may normalize to absence); MySQL NULL COLUMN_DEFAULT on nonnullable column is None |
| Exact integer literal default | Checked parse for each native width/sign; PG finite integer deparse forms only; no floats, coercion, general expression parser or evaluation |
| PG bool default true/false | Exact Bool literal; missing definition remains Unknown |
| PG missing default definition / MySQL NULL default on nullable column | Unknown: original absence vs normalized explicit NULL cannot be established; never normalize that ambiguity to None |
| Retained SQL NULL/text/other default expression outside that finite grammar | Unsupported; opaque source strings cannot prove semantic correspondence |
| View / non-base relation | Fixed Unsupported refusal |
| Trigger, PG RLS/forced RLS, generated/identity/auto-increment/on-update, collation-bearing column, unsupported native type | Unsupported, never ordinary Complete |
| Foreign/check/exclusion constraint, inheritance/partitioning/rewrite, partial/functional/include/invalid index or non-builtin/nondefault PG operator class | Unsupported |
| MySQL prefix/expression/descending/invisible/non-BTREE index, generated/default-generated/zerofill column, non-InnoDB table | Unsupported |
| Missing or insufficient visibility | Unknown/refusal; empty result is not a Complete absence |

Only PostgreSQL server_version_num 160014 and MySQL VERSION 8.4.8 are supported by
this measured subset; other versions refuse. MySQL also requires visible generated
invisible primary-key metadata and lower_case_table_names=0; contrary settings refuse.
PG checks schema USAGE and whole-table SELECT. MySQL requires **explicit
schema-wide SELECT and TRIGGER grants** to the actual CURRENT_USER grantee, so
trigger metadata cannot silently disappear. Column/table-only or inherited-role
privileges are conservatively Unknown even if a server would permit some reads.
TRIGGER is a metadata visibility prerequisite here, not permission used by the
reader to create anything. Restrict application access to the reader API and use
server read-only transaction enforcement. This finite support does not claim
unread extensions, arbitrary server semantics, defaults or SQL type families.

## Budgets and transaction lifecycle

`Budget` is explicit: catalog SELECT submissions 1..65 (plus at most two fixed
transaction-control submissions), total rows 1..8192, semantic encoded bytes
1..1,048,576, each decoded string 1..4096 bytes, tables 1..16, columns/table 1..256,
keys/table 1..16, comparison branches (table branches) 1..256, operation timeout
0 < timeout <=60s. At most 16 components/key. Metadata SELECTs have fixed LIMIT
2 (identity/header) or 257 (column/index-component sentinel); a sentinel row always
fails, including nonunique index rows. Variable catalog strings are SQL-clipped to
4097 characters, then byte-checked and refused, never accepted as truncated data.
All rows/strings, including ignored index fields, consume budgets. SQLx protocol
buffering/server work is outside the semantic retention budget; the finite query
shape, server-side projections and timeout bound the supported reader's workload.
There is no retry or silent partial-success fallback.

PG begins `REPEATABLE READ READ ONLY`, providing a catalog MVCC snapshot for the
measured SELECTs. MySQL begins `READ ONLY`; information_schema does **not** promise
one coherent MVCC DDL snapshot. MySQL correspondence is conditional on the caller
excluding concurrent DDL for the observation window. Neither reader locks user
tables, verifies an external DDL exclusion mechanism, authenticates standard catalog
implementation, or guarantees absence of DDL after observation. MySQL's default
isolation/metadata behavior is not represented as PostgreSQL snapshot semantics.
These limitations apply even to Matched and unexpired evidence. A Complete
observation must also pass the closed v1 shape validator (identifier grammar,
resolved nonempty keys, exact literal types); unrepresentable catalog metadata
never receives Complete support.

All complete read paths explicitly await rollback (there is no commit/business
write). On error, explicit rollback is attempted within the same total timeout.
Observation is minted only after complete reads **and acknowledged rollback**.
Any polled failure/cancel/timeout quarantines that reader; no reconnect/reuse API
exists. SQLx transaction Drop only queues best-effort cleanup and is not reported
as acknowledged rollback. An unpolled observe future does nothing. Dropping a
reader closes its connection; disposable fixture cleanup is separate test evidence.

## Comparison, provenance and time

`observe(current, scope, source, budget, ttl)` privately constructs a non-Clone,
non-Serialize/Deserialize Observation only on real successful I/O. It retains exact
context (all identities, independent generations and content), private connection
allocation identity, scope, complete input source, observed finite schema,
observation start/completion and deadline. A new connection with identical labels
cannot recheck old evidence. Matching generations cannot replace matching contents.
Source/policy/code/external live records never become observation constructors.

Monotonic `std::time::Instant` is authoritative in process; TTL starts before reads,
0 < ttl <=300s, checked deadline addition, exclusive expiry `now >= deadline`.
Completion after expiry refuses. Compare checks current target/context/source/scope
and time again, including after canonical/drift work. Backward monotonic samples,
wall time before Unix epoch, millisecond conversion overflow or backwards wall time
fail closed. Wall timestamps are sampled from SystemTime as checked u64 Unix
milliseconds; they are descriptive, private and not a persistent freshness source.
Forward wall jumps do not extend monotonic TTL. No public clock injection, restore
or cross-process Fresh cache exists. Persisted JSON wall time/generation/offline
labels cannot recreate Observation. Supplying obsolete application snapshots is
still an application responsibility.

`reader.compare(current, scope, source, observation)` returns output-only
`quent_verify_summary` version 1: schema verdict, combined expected/observed support coverage, five bounded drift
counts; policy/generated_code/external_live are always Unverified. Matched requires
Complete support on both sides and exact **full canonical schema bytes** after
sorting tables and keys as above. Kind/version, full target, coverage, every column
and default, ordinal order and ordered key components participate. PR1 offline
canonicalization preserves all source array order; that contract is unchanged.
FNV hints and generation labels never decide equality. Unsupported identical strings
stay Unsupported. Unknown/Partial never become Matched. Fixed difference counts
are table set, column set/order, native, null/default and ordered-key differences;
they are a bounded description, not an exhaustive migration plan. Sensitive names,
old/new values, time, generation, fingerprints and SQL are omitted.

Missing schema, external/unbound evidence, stale, Unknown/Partial, Unsupported,
measured Drift and Matched have separate classifications. Failed reads return a
closed VerifyError and no evidence. Debug/Display/JSON/errors/source expose only
fixed codes, enum classifications and bounded counts. No exporter/tracing trait
accepts source, reader or Observation. This evidence grants no business authority,
DB exactly-once or new execution path. Existing Strict gates remain unchanged.

The [closed output schema](manifest-verify-v1.schema.json) describes both JSON kinds;
it is documentary output only, never an observation or authority parser.

## CLI grammar and output/exit register

```
quent-cli manifest verify --offline SOURCE COMPARISON
quent-cli manifest verify --live SOURCE CONNECTION_CONFIG
```

Exactly these positional arguments; unknown/extra/missing arguments refuse before
file reads. Source files are regular files, <=65,536 bytes, strict production
ManifestSource parser, finite read of max+1 bytes. Paths <=4096 encoded bytes.
Offline opens only the two explicitly named source files and never connects.
Offline success means complete supported schema, Complete finite source coverage
for every other present artifact, and full exact source canonical equality, labelled OfflineConsistent with live Unverified; it verifies no code,
policy authority or live claim. Missing schema or any present Unknown/Partial/Unsupported artifact never exits 0.
Other absent slots remain missing; no facts are fabricated to fill them.

Live config is UTF-8 strict JSON <=8192 bytes; duplicate/unknown fields refuse.
All fields below are required; no DSN, environment lookup, default target, SQL or
password argv is accepted. Store credential-bearing config as sensitive local data.

```json
{"transport":"tcp_plaintext","dialect":"postgres","host":"127.0.0.1","port":5432,"user":"reader","password":"","database":"application_db","address":"127.0.0.1:5432","catalog":"catalog_name","namespace":"public","tables":["items"],"generation":1,"timeout_ms":5000,"ttl_ns":30000000000}
```

Dialect postgres/mysql; host is a bounded explicit TCP host, port nonzero u16;
user nonempty <=128 bytes, password <=4096; generation nonzero u64; timeout is in milliseconds; TTL is in nanoseconds and
must satisfy the monotonic limits above. Empty MySQL password explicitly
means no password (SQLx's empty-string scramble is not substituted). The example
selects plaintext explicitly: CLI TLS/socket transports are unsupported; library
callers may supply explicitly configured SQLx TLS options under the existing feature
boundary. No TLS/certificate behavior is claimed by this CLI subset.

Live validates all target/scope/config fields before connection. Because pinned
SQLx PG constructors consult PG* defaults, live CLI invokes the same executable
after clearing the child environment and adding only fixed, nonsecret PGHOST/PGPORT/PGUSER
bootstrap constants, transferring bounded source/config bytes over
stdin, never argv. No process-global environment mutation occurs. These three constants prevent SQLx constructor whoami/default-socket discovery and
are all overridden by validated explicit config before connection. The internal
worker transport accepts exactly that closed bootstrap environment and refuses
other ambient settings or extra args; it is not a public
source exporter. Parent maps abnormal child status to fixed non-success 8.
Inherited PGOPTIONS/password/certificate paths cannot configure the live worker.
The library options API retains the documented application-owned construction duty.
No environment service/credential discovery occurs. No live connection starts
without the explicit --live workflow and explicit config.

| Exit | Meaning / fixed stderr codes (stdout, when present, is only allowlist JSON) |
| --- | --- |
| 0 | OfflineConsistent source comparison or live Matched, distinct JSON kinds |
| 1 | Existing unrelated unimplemented CLI routes: QUENT_NOT_IMPLEMENTED |
| 2 | Invalid/target/budget: QUENT_VERIFY_INVALID / TARGET / BUDGET |
| 3 | Missing/unverified/unknown: QUENT_VERIFY_MISSING / UNKNOWN, or fixed summary |
| 4 | Feature/semantics unsupported: QUENT_VERIFY_UNSUPPORTED or summary |
| 5 | Offline Different or live Drift summary |
| 6 | Expired/clock anomaly/Stale: QUENT_VERIFY_STALE or summary |
| 8 | I/O/connect/runtime/abnormal child/output failure: QUENT_VERIFY_IO / CONNECT |

The fixed panic hook emits QUENT_PANIC without payload/location/backtrace; child
abnormal exit becomes 8. No arbitrary driver/OS/serde error chain, identifier,
credential/DSN, path, SQL, default, name or value is formatted. Other previously
unimplemented CLI functions keep their refusal. Default CLI recognizes verify's
finite grammar only to return feature Unsupported, without file/DB dependencies.
Windows protected local export remains UnsupportedOs; no ACL feature is added.

## Evidence and remaining checks

Production tests are in `manifest/live/tests.rs`, CLI process tests in
`quent-cli/tests/manifest_verify.rs`, and diagnostic-checked nonpromotion controls
extend the existing sequential `owned_api.rs` harness. `scripts/test-dispatch.sh`
creates only new digest-pinned PostgreSQL 16.14 / MySQL 8.4.8 containers and removes
exactly their containers/anonymous volumes, checking absence even after failure.
Reader/business roles remain quent_test; quent_manifest_hidden separately tests
insufficient metadata visibility. PG catalog fixtures use public.manifest_probe;
MySQL uses quent_dispatch_test.manifest_probe and admin-created
manifest_trigger_probe (binlogging forbids limited-role trigger setup). No SUPER
privilege is granted to the reader and no global binlog setting is changed.
Alternate same-basename fixtures use PG manifest_other.manifest_probe and MySQL
quent_manifest_other.manifest_probe with a deliberately different literal default.
PG finite Complete examples therefore use explicit integer/bool literal defaults on
every column; source None claims cannot establish a PG live match when normalized
NULL cannot be excluded. MySQL absent/explicit NULL on nullable columns is tested
as Unknown as well.
CLI fixtures use cli_manifest_probe in the same disposable namespaces. Existing
CRUD/validated/atomic suites keep their original isolated table/schema and settings.

Real catalog measurements and synthetic/private clock checks must be distinguished
in the execution/PR report. A cfg(test) fixture hook can terminate the actual DB
connection after two acknowledged catalog SELECTs, then observes a real downstream
read failure; fixture termination control is recorded separately from catalog SQL.
This is not a deterministic network proxy or lost rollback-ack test. DDL races,
physical server authentication/replacement, arbitrary TLS, broader permission
models/server extensions and lasting post-transaction freshness remain unverified.
All 24 full fixture runtime_pending markers remain; finite C19/C24 with C08/C12/C15
and existing CRUD/owned/atomic regression do not clear whole-case acceptance.
Commands/results/failures/skips, exact commit/CI and actual cleanup belong to the
execution report, never an inferred PR number or source-declared future head.

## Local measurements for this revision

WSL2 Ubuntu 24.04.5 / Rust 1.94.0: final full workspace/all-feature regression on
both new digest-pinned databases passed 131 tests, zero failed/ignored/filtered
(126 retained plus three reader tests and two CLI tests). The shared real compiler
harness retained 100 negatives + five controls and now asserts 127 diagnostic-checked
negatives + six compiling controls. Final full and DB-free harness runs were sequential.
DB-free `--skip live_` passed 124 tests with seven exclusions; the two accidental
substring exclusions (Core current-context and manifest live-claim nonpromotion)
were each run separately and passed. The other five exclusions are the two business
DB suites, two catalog suites and one both-dialect live CLI suite, covered by full.
Default workspace remains 81 passed. No ignore/no-op substitutes for DB evidence.

Final fmt, default/all-feature build, warning-denied all-target/all-feature clippy
and rustdoc passed. The original 32 facade driver/manifest checks, 16 additional
CLI feature checks, manifest-only CLI tests and all seven output/manifest test+
clippy combinations passed. Core's normal tree is empty; default facade and CLI
normal trees contain only Core. Manifest-only facade adds serde/serde_json and their
locked transitives, no driver/runtime. All exact version/toolchain/edition/publish
pins are unchanged. Cargo.lock only attaches five already-locked optional CLI
dependencies. runtime-tokio enables Tokio time for the finite reader deadlines.

PG measured version/settings: 16.14, UTC, read committed default (reader explicitly
uses repeatable read read only), UTF8, en_US.utf8. MySQL: 8.4.8, +00:00,
REPEATABLE-READ, utf8mb4/utf8mb4_bin, existing strict SQL modes and FOUND_ROWS;
reader additionally checks generated-invisible-PK visibility and lower-case mode.
Both final containers and all their anonymous volumes were removed and absence
verified. Only the script-created disposable catalogs/namespaces/roles were used.

Development failures were corrected rather than treated as evidence: PG functional
index metadata and NULL-default normalization; MySQL binlog restrictions on unsafe
expression/limited-role trigger setup and text-default syntax; SQLx empty-password
scramble; initial test build errors; and a local disk-full build. Only repository
Cargo artifacts were cleaned, and local final commands used CARGO_INCREMENTAL=0,
CARGO_PROFILE_DEV_DEBUG=0 and CARGO_PROFILE_TEST_DEBUG=0. No host/network/global DB
setting or dependency pin was changed to force a pass. Hosted Ubuntu/Windows and
same-head CI results are separate evidence recorded in the PR/execution report;
local Windows/live TLS/physical identity/concurrent-DDL guarantees remain unverified.
