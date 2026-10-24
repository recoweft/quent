# Contract fixture and recorder support (test-only)

`cargo test -p quent --test contracts --locked` runs the source validator,
coverage/negative tests, recorder unit scenarios and two dialect consumers.
The workspace CI test step runs these on Ubuntu and Windows with default features;
no driver, Tokio, DSN, network or server is needed. This directory is an integration
test module, never a library module or public CLI/MCP/tracing surface.

## Wire contract

`fixtures/contracts-v1.json`: kind `quent-contract-fixtures`, version string `1`.
This is a language-neutral test source, not an OperationSpec, executable public
plan, trusted context, permit, or Go wire compatibility layer. No Go native int,
legacy risk result, or permitted artifact behavior has been copied as authority.

The loader accepts exactly one UTF-8 JSON document. All objects reject unknown
and duplicate fields (including nested enum objects). Every declared field is
required. JSON numbers and JSON null are rejected: integers, versions and counts
are strings, and SQL NULL has an explicit typed object. Budgets: 256 KiB/document,
4096 UTF-8 bytes/string, 128 bytes/field name, 64 fields/object, 256 elements/array,
48 structural levels (root zero), 16 condition edges; serde_json's additional
parser recursion limit remains enabled. Byte budget bounds allocations before
parsing; structural validation runs before typed decoding. No unbounded stream,
multiple document or old-version fallback. The fixed v1 register requires all
24 unique IDs and an independent exact owning-issue map.

`shape` preserves operation, explicit projection, ordered input column names,
ordered typed rows, condition tree and documentary scenario steps. AND/OR child
order, NOT, NULL comparisons versus IS NULL, and empty IN remain distinct. The
loader does not normalize predicates or infer SQL truth. Each row's width must
match input columns. Missing, SQL NULL, empty text, integer zero, false are
separate variants. Signed/unsigned 8/16/32/64-bit integer lexemes are range checked
using exact i128/u128 without normalizing the lexeme. Decimal lexemes use optional
minus and digits with optional decimal point; precision 1..65 counts all digits,
scale is the exact fractional digit count, scale <= precision. No exponent,
float, rounding or driver encoding. This finite vocabulary deliberately excludes
UUID/bytes/date/time until explicit versioned representations and tests are added.
Schema is documentary base-table metadata; supported type names are finite and
key references must name known columns. It does not establish live schema facts.

`preconditions` separates schemas/targets/generations, policy, tenant provenance,
typed tenant and freshness/allow claims. Even "application assertion" in a fixture
is just text, never an authenticated context. QT-C08 deliberately has allow=true
and a stale freshness claim; the loader does not issue authorization. Steps and
expected diagnostic names are contract descriptions, not production diagnostic
code registrations. Expected risk/precision, dispatch/provider counts, Tx and
outcome are separate. `scenario_dependent` is an explicit unresolved count, never
zero or a measured result; its runtime owner must refine it into actual observed
substeps. Transaction strings are expectations, not observed events.

## Evidence and extension

Evidence vocabulary: kind `quent-contract-evidence`, version `1`.
Each case has exactly `documented`, `fixture_validated`, `runtime_pending` in that
order. The first means described, the second means source validation, and the
third means runtime acceptance absent. `recorder_tested` applies to support tests
only. It is not attached to all 24 runtime cases. No test invokes absent production
APIs and claims a runtime pass; no fake compile-fail test fails for a missing API.

Add a subscenario under its stable ID with prerequisites, typed data, positive/
negative behavior and explicit unverified limits. Keep owning issues synchronized
with specs/acceptance.md and the independent owner register. Extend the finite
vocabulary with validator and rejection tests; incompatible changes need a new
version, never permissive fallback. `load` rejects unsupported evidence claims.
Future runtime evidence must be recorded separately from source expectations.
QT-C21/22/23/24 owners remain QT-AI-12 / QT-AI-11 / QT-AI-13 /
QT-AI-05+06+13 respectively; all four are fixture validated, runtime pending.

## Recorder

The recorder accepts only finite Begin/Call/Commit/Rollback events; calls have an
operation, dialect, finite Probe or Fixture descriptor, and ordered typed args.
Missing is not a bound argument. Scripts configure rows, a dialect-specific count,
finite errors, Unit for Tx events, or Unknown. PostgreSQL affected and MySQL
changed/matched/upsert counts stay distinct; no cross-dialect equality is inferred.
There is no raw execution API, connection or SQLx Executor implementation.

Every attempted event and returned result is logged in order. Wrong order/args,
extra calls, missing scripted calls, invalid Tx state and result kind are rejected
or fail exact `verify`. Clone makes an independent deep snapshot of script/log/Tx
state. Reset clears all three. Tests include changed clone/reset, failure followed
by explicit rollback, unknown and failed commit, unobserved Tx, ordered rows/args,
call counts and negative verification. Unknown never becomes success; failed
commit/rollback makes Tx outcome unknown. A recorded rollback proves only that a
mock event was observed. An application must never infer real DB success from it.
Synthetic fixture descriptors in the template test deliberately return Unknown.
No normal production output is added; synthetic source data is safe to check in.
Do not place credentials, real tenant data or DSNs in fixtures or assertion output.

## PostgreSQL / MySQL templates and future real DB tests

`dialect_template` consumes the same case/scenario records for each dialect and
compares semantic expectations, not literal SQL equality. Placeholder/quoting,
row count behavior, RETURNING and DDL capabilities require separate adapters and
real server evidence later. This PR has no SQL formatter/planner/dispatcher.
The template test is source-consumer coverage, not DB integration.

No real servers were used; server versions/settings/images remain unselected and
unobserved. Real DB tests must be separately opt-in (e.g. QUENT_TEST_DB=postgres
or mysql plus an explicit QUENT_TEST_POSTGRES_DSN / QUENT_TEST_MYSQL_DSN). Never
fall back to DATABASE_URL, production credentials or a default local service.
Absence of opt-in/DSN/capability must produce an explicit skip reason in the test
report; invalid selected configuration is a failure, not silent success. There is
no ignored or no-op runtime test in this PR that could imply acceptance.

Before connection, the future harness must validate an explicit disposable test
target and capability requirements. Create a unique isolated PostgreSQL schema
with fixed search_path, or a unique MySQL database (schema synonym); do not assume
MySQL schema isolation inside one database. Test credentials must be restricted
to that disposable namespace. Record server version, collation, time zone,
SQL modes, MySQL found-rows setting, transaction isolation and relevant trigger/
RLS/capabilities without credentials. Pin the selected server image in that PR.
The harness owns cleanup even on assertion/driver failure; never drop a shared
namespace or claim MySQL DDL rollback. Report cleanup failures and unknown commit
outcomes independently. Gate/tenant/row decoding/Tx/command/canary and compile-fail
acceptance remains with the owning issues, including both dialects where needed.

## PR2 local verification

Ubuntu 24.04.5 / WSL2, Rust 1.94.0: 13 integration tests passed, 0 failed,
0 ignored; no production unit/doc tests are present. All 24 cases / 66 scenarios
validate. fmt, workspace build/test, all-target/all-feature clippy with warnings
denied, all-feature rustdoc with warnings denied and each driver compatibility
check passed using --locked --offline. No real DB test was run; no server
version/settings or GUI behavior was checked. Windows execution is checked by
the CI workspace matrix, separately from these local results.

Only dev-dependencies serde =1.0.229 (MSRV 1.56), serde_json =1.0.154
(MSRV 1.71), and serde's derive feature (serde_derive MSRV 1.71) were added for
strict typed test parsing. Their exact versions already existed in PR1's lock;
the lock diff only attaches serde/serde_json to quent. Compilation on pinned
Rust 1.94.0 passed. Core still has no dependencies and the default normal
workspace tree has neither SQLx/Tokio nor serde. No production dependency,
workspace member, toolchain, default feature or publish=false setting changed.

## QT-AI-01 PR1 addition

The production planner now consumes selected QT-C01/QT-C07 inputs in `contracts.rs`
through an explicit test-only finite bridge. Independent SQL/args expectations
are checked for both dialects; decimal/missing/overflow are refused. No fixture
verdict becomes risk/authority. Recorder observations remain empty because core
has no DB/provider endpoint. This is plan evidence only; full runtime acceptance
stays pending. See [capabilities](../../../../specs/plan-ir-v1.md).

## QT-AI-01 PR2 inspection consumer

The production bridge in contracts.rs consumes QT-C02–05/C07 (12 scenarios × both
dialects), independently compares scope/risk/ordered args, and checks unchanged
SQL/tree/metadata. Three scope-only positive/finite OR source scenarios bring the
total to 69. It first checks UnsupportedContext, then supplies a separate explicit
application assertion of ordinary base-table semantics; schema/fresh/allow text is
never key/context authority. Nullable text unique is outside the supported signed-key
proof domain. QT-C07 lossless-values is skipped by this bridge and remains covered
by the separate value-refusal test. See ../../../../specs/scope-risk-v1.md for finite
rules and differences from full future policy/Tx fixture expectations. Recorder
observations remain empty; no dispatcher or DB behavior is claimed. All runtime
acceptance markers stay pending.
