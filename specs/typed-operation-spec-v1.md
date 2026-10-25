# QT-AI-06 planned PR2: typed read compilation

The optional `operation-spec` feature remains driver/runtime-free. Core has zero
dependencies; default facade and CLI depend only on Core. All existing exact pins,
lockfile, edition 2024, Rust 1.94.0 and publish=false remain unchanged. Source v1
and all PR1 grammar/budget limits remain unchanged, including required limit,
nonempty IN/AND/OR and rejection of unknown tags before compilation.

## Application contract and carriers

Authenticated, authorized server code supplies `TypedColumn::application_supplied`
and `TypedModelContract::application_asserted`: one model symbol, an exact existing
Core two-component Table without alias, unique symbol/physical Ident mapping,
native type/sign/width/nullability, Ordinary/Tenant/Unknown role, read restriction,
Complete/Unknown coverage and complete DatabaseContext. Physical columns must be
canonical and unique, even under different source symbols. An asserted Ordinary
role cannot conceal a physical Tenant policy column. All registered columns must
match the complete ordinary-table registry in the existing schema snapshot, including
unused columns; missing, unsupported, unobserved, unknown coverage and mismatched
mapping never silently establish typed success. The table/column declarations are
application assertions, not live authentication or guarantees against concurrent DDL.

`SpecSource::compile(&mapping, &current)` first compares full snapshot contents,
checks coverage/native support, resolves every projection/order/filter occurrence
(including NOT and dead OR branches) and applies PR1 role checks. All tenant
predicates refuse, even equal values and physical aliases. Source values never
become context. The old role-only ModelContract cannot compile by itself.

Then the compiler builds a SELECT Draft and calls existing common preparation
once. Core appends tenant and soft-delete guards outside the entire caller filter,
enforces PII/RequiredFilter/coverage, and finalizes once with its existing renderer.
There is no second SQL renderer, binder, scope/risk policy or missing-WHERE rewrite.
Limit is a native bound i64 argument, never scope or authorization evidence.
Compile/inspection/diagnostics use no DB, provider, catalog, EXPLAIN, clock,
filesystem, environment lookup or mutable global. Current context is explicit.

Syntax source, role-checked source, typed CompiledRead, public PlanView and existing
validated/atomic handles remain separate carriers. CompiledRead privately owns the
complete source, mapping, context and PolicyPreparation. Inspection/PlanView borrow
that preparation's final record. Private carriers have redacted Debug and no
Serialize/Deserialize or public restoration/conversion/operation-replacement API.
Existing inspection's explicitly sensitive source boundary still requires local
handling; do not log its SQL/args/names. Public JSON cannot reconstruct authority.

## Native, dialect and operator table

| Application native type | PostgreSQL | MySQL | Non-NULL predicates |
| --- | --- | --- | --- |
| I8 | refuse: no supported native PG width | exact signed TINYINT | Eq/Ne/Lt/Le/Gt/Ge/IN |
| I16/I32/I64 | exact INT2/INT4/INT8 | exact signed SMALLINT/INT/BIGINT | Eq/Ne/Lt/Le/Gt/Ge/IN |
| U8/U16/U32/U64 | refuse: unsigned unsupported | exact unsigned corresponding width | Eq/Ne/Lt/Le/Gt/Ge/IN |
| Bool | existing native bool | existing finite BOOLEAN subset | Eq/Ne/IN; ordered comparison refuses |
| Text | existing text subset | existing nonbinary character subset | Eq/Ne/IN; ordered comparison refuses; NUL refuses |
| Decimal/UUID/Date/Time/Bytes/Float/Other | explicit unsupported family | explicit unsupported family | refuse, including typed NULL |
| Unobserved | refuse | refuse | no default-safe type |

Literal tags must exactly match column native types. Conversion checks native
ranges and never casts, rounds, widens, trusts a string type name or uses float.
MySQL integer width/sign are retained in bind and decode. PR1 independently rejects
all overflows, float/exponent/string-number spellings and unsupported source tags.

| NULL / tree form | Typed behavior |
| --- | --- |
| Eq/Ne/Lt/Le/Gt/Ge typed NULL, including inside NOT | refuse; no implicit rewrite to IS NULL |
| typed NULL on nonnullable column | refuse |
| IN with typed NULL | nullable column and exact element tag required; retain NULL and order, Core SQL UNKNOWN semantics |
| mixed-tag IN | refuse every mismatch, never coerce |
| IS NULL on nullable/nonnullable column | preserve explicit predicate, no fold |
| NOT / AND / OR | retain tree and validate every branch, no dead-branch shortcut |

These are compile capabilities. Existing scope inspection is deliberately narrower:
nullable/text/bool/NOT/IS NULL and soft-delete may yield Partial/Unknown/High and
therefore remain refused by the execution gate even when type compilation succeeds.
A typed success is never business permission. No new native family is added.

## Fixed output register

`quent_typed_spec_diagnostic`, version 1, state `refused`, contains only fixed
`code` and `action`. Serialize-only, independent from source, old syntax diagnostics
and PlanView. No names, values, SQL, path, free reason, driver/serde error, source
fingerprint or arbitrary text enters Debug/Display/Error/source/JSON/tracing.

| Code | Action |
| --- | --- |
| mapping, unknown_coverage, unobserved, role | check_application_registration |
| unsupported_family, dialect, operator, native_refused | check_native_support |
| type_mismatch, non_nullable_null | check_literal_type |
| null_comparison | use_explicit_null_predicate |
| read_denied, tenant_predicate, policy_refused | check_read_policy |
| snapshot_mismatch | recompile_current_snapshot |

Actions are suggestions for an explicit application decision. They embed no SQL
or values, do not autocast/patch/remove tenant guards, and cannot execute or restore
permission. PR1 source diagnostics keep their independent closed version-1 register.

## Application-only read and same record

With a driver and runtime feature, existing owned PostgresSession/MysqlSession
provide `execute_compiled_read(&current, &current_mapping, owned_compiled)`.
Connection is still only via explicit existing connect. There is no borrowed/atomic
spec extension, public SQL/executor callback/connection getter or new validation
handle type. A future consumes the artifact even if dropped without polling.

At execution, exact current mapping and all retained context content must match,
including target, subject, tenant, policy/schema/settings independent generations
and same-generation different contents. Any substitution needs a new compile.
The current immutable inputs are borrowed throughout the future. The retained
preparation enters the same private Strict risk/completeness/subject/capability gate,
then the existing session-bound consumed validation carrier and common dispatch
with the same exclusive 300-second monotonic expiry and final identity/context
checks. No operation is rebuilt and no SQL is rendered again. The optional internal
operation comparison only permits the retained original for compiled reads; ordinary
and atomic CRUD retain their explicit current-operation comparison.

Shared native binding/decode budgets remain 256 rows, 4096 bytes/text and 1 MiB
semantic result payload. Errors/canceled submission quarantine the owned session;
no retry. Results use existing redacted ExecutionResult and explicit sensitive
accessors; public result summaries contain no rows/count/affected values. Compile
has no deadline/clock; the execution deadline belongs to the existing runtime gate.

## Evidence scope and limits

`tests/typed_spec.rs` covers both dialects, independent SQL/native args/metadata,
repeated determinism, all native width/sign extrema, nullable NULL/NOT/IN/operator
and mixed-tag refusals, complete mapping and content mismatch, all-branch tenant
refusal, unsupported/unobserved declarations, PII/RequiredFilter and source-free
fixed diagnostics. Existing PR1 tables retain empty/missing/overflow grammar tests.
The production live suites add actual compiled read/native+NULL results, independent
SQL/args expectations and exact inspected SQL allocation correspondence at dispatch,
plus current context/mapping and partial/high refusals with zero additional sends.
An inaccurate nonnullable application declaration for an actual NULL result is
refused by native decode after one submission; the session is quarantined and
subsequent read attempts send nothing.
The shared compiler harness retains its original 153 negatives/seven controls and
adds 17 typed carrier/output/restoration/write-substitution/reuse refusals (170
total negatives) and a real
compile/inspect/execute API positive control (eight controls total). Existing CRUD/validated/atomic/catalog,
CLI FIFO three-entry, byte/OS and driver-output canaries regress unchanged.

Measured commands, counts, failures/skips, actual new-head hosted CI, server settings
and disposable cleanup belong to the execution/PR report. Source fixtures alone are
not dispatch evidence. All 24 full runtime_pending markers remain unchanged.
No general physical-target authentication, DDL-race protection, TLS handshake,
Windows live verification or OS-wide I/O response deadline is added. MCP, CLI/review
workflows, natural language, write specs, command, migration, codegen and EXPLAIN
remain outside this PR. No Issue closure, merge, release or publication is performed.


## Local measurements for this revision

Ubuntu 24.04.5 / WSL2, rustc/cargo 1.94.0. Final full workspace/all-feature tests
passed 153, zero failures/ignored/filtered: all original 145 plus eight typed tests.
Default remains 81; DB-free `--skip live_` passed 145, with the three accidentally
matching DB-free tests (Core context, manifest live-claim nonpromotion, CLI config
zero-connect detecting control) run separately. The five actual live suites run in
the full regression. The shared real compiler harness ran sequentially and passed
170 checked negatives plus eight compiling controls. Product CLI all-feature build
and complete test compilation preceded the final sequential full run.

Fmt, default/all-feature build, warning-denied all-target clippy/rustdoc, all 80
feature checks, all 12 output test/clippy combinations and dependency trees passed.
Core has no normal dependencies; default facade/CLI have only Core. Operation-spec
alone adds pinned serde/serde_json and locked transitives without driver/runtime.
All manifests, Cargo.lock, toolchain, CI jobs and documentary fixtures are unchanged.

Final new disposable PostgreSQL 16.14: UTC/read committed/UTF8/en_US.utf8; MySQL
8.4.8: +00:00/REPEATABLE-READ/utf8mb4/utf8mb4_bin, existing strict modes and
FOUND_ROWS. Both retain the exact image digests in `scripts/test-dispatch.sh`.
Only disposable catalogs/namespaces/roles/settings were used; final containers and
anonymous volumes were removed and absence checked. Earlier development failures
were Rust macro scope/Option call-site errors and an incorrect expected SQL
qualification/parenthesization, corrected before these final runs. Hosted CI must
be checked against the actual new commit, independently of local measurements.

## QT-AI-07 planned PR1 code artifacts

[Finite typed Rust generation](typed-codegen-v1.md) adds optional pure codegen,
model-specific keys/columns/predicates/order/projection/results/repositories and
explicit application mapping/context binding through the existing read compiler
and same finalized-record dispatcher. Source metadata, generated Rust and public
summary/diagnostic JSON remain separate untrusted/output-only domains. No patch,
regen workflow, new write/provider, authority restoration or risk relaxation is
introduced. Prior PR absence statements above are historical. All 24 full
runtime_pending markers remain unchanged; actual new-head evidence is reported
separately.
