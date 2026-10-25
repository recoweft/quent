# QT-AI-07 planned PR1: finite typed Rust code artifacts

`quent/codegen` adds a pure generator API, not a CLI regeneration workflow. It
includes the existing optional `manifest` and `operation-spec` features and their
exact serde/serde_json pins; it adds no package, dependency version, driver or
runtime. Core stays dependency-free, default facade/CLI Core-only. Workspace
Rust 1.94.0, edition 2024, SQLx 0.9.0, Tokio 1.54.1, futures-util 0.3.34,
serde 1.0.229, serde_json 1.0.154, tracing 0.1.44, rustix 1.1.4, lockfile and
publish=false remain unchanged.

## Input, canonical domains and support

`codegen::generate(manifest_bytes, config_bytes)` accepts existing strict manifest
source v1 and independently versioned `quent_codegen_config` v1, generator
`quent-codegen/1`. Both inputs use the existing production streaming duplicate-key,
UTF-8, integer, depth, node, string, object, array and 65,536-byte bounds before
serde tree construction. Config structs deny unknown fields at every level.
There is no fallback receiver. Version/generator mismatch, unknown fields/tags,
duplicate decoded keys, overflow, float/exponent, missing required fields and
budget excess refuse with fixed errors. Parser failures do not retain error text.

Finite config wire (all fields required):

```json
{"kind":"quent_codegen_config","version":1,"generator":"quent-codegen/1","models":[{"table":["app","items"],"columns":[{"name":"id","role":"ordinary","read":true}],"projections":[{"name":"detail","fields":["id"]}]}]}
```

This config requires a corresponding complete manifest schema and policy. Config
models/tables and columns must exactly cover the schema arrays **in their original
order**, with identical physical names. There are no symbols/aliases to infer.
`role` is ordinary/tenant/unknown, and unknown refuses. Tenant role must exactly
match the corresponding finite policy declaration. `read` is an additional
application assertion to compare at bind, never a business allow. Every projection
field must be distinct, present, readable and ordered explicitly.

| Source or mapping | PR1 handling |
| --- | --- |
| Complete ordinary table, exact native/null/default/key declarations | finite generation only; never live/trusted facts |
| Missing schema/policy, Unknown/Partial coverage, missing/unknown default or native/nullability | refuse |
| Views, triggers, RLS, collation, generated table semantics, unsupported/default expressions | refuse; never normalized to Ordinary/Complete |
| Exactly one complete primary key; additional complete unique keys | preserve all key kinds/component order in bind comparison; nominal Key uses primary order |
| Missing/partial key, nullable/unsigned/text/bool unique or primary component | refuse under existing manifest proof subset |
| Different schema/policy targets, dangling/duplicate policy columns | refuse |
| Source generated_code/live_observation slots present | unsupported for this generator input; no claim promotion |
| PII/RequiredFilter/readonly/generated policy columns on ordinary source tables | preserve exact declarations; common preparation/gate remains final enforcer |

The source manifest remains source data. No JSON role, read, tenant, allow, key,
coverage or generated-code claim restores ScopeFacts, SchemaSnapshot, subject,
tenant context, permit or a live observation. The app independently supplies its
current immutable context and full typed mapping. Internally generated comparison
templates are compared to those assertions and never returned as authority.

Canonical `quent-rust-code/v1` is emitted directly as UTF-8 with LF and a final LF,
without rustfmt, clock, catalog, provider, filesystem, network or environment input.
Its artifact kind is `quent_rust_code`, version 1.
The first fixed comment names the generator/artifact version. Three anonymous
private string constants retain exact ordered canonical schema-v1, policy-v1 and
config-v1 bytes (config struct field order in codegen.rs). Manifest canonicalization
is unchanged. Table/column/key/component/rule/projection order and target identity
are never sorted or erased. Each model follows in input order. Code emission has
no mutable globals. Exact repeated inputs produce identical source bytes. Full
canonical content, not a fingerprint, defines correspondence. No fingerprint is
needed or exposed. Source identity, schema, policy, generator and emitted code
remain separate domains; generated code is UnverifiedCode, never liveFresh/permit.

## Naming and budgets

Physical path components, model module names, columns and projection names must
be exact lower ASCII snake identifiers, 1..63 bytes, starting with a lowercase
letter, with no trailing underscore or consecutive underscores. Rust strict,
reserved and edition-2024 keywords and a conservative fixed implementation-name
register refuse. Quotes/backticks, Unicode, uppercase/case conversion, digit-first,
path separators, raw identifiers, aliases and code fragments refuse. No Unicode
normalization, suffix repair or physical name folding is performed. Module name
is the exact table basename: same basename in different namespaces refuses.
Projection row names use underscore-to-Pascal plus Row; collisions after this
conversion (including `a1`/`a_1`) refuse. Projection factories cannot collide with
column factories. Physical columns otherwise retain their exact distinct names.
All inserted target/canonical strings are Rust escaped string literals, never tokens.

Additional pre-emission bounds: 1..8 models, 1..32 columns/model, 128 total columns,
1..8 keys/model (existing max16 components/key), 1..8 projections/model,
1..32 fields/projection, 2048 generated items (8 per column + each key + each
projection and selected field). Emitter checks each append against 524,288 code
bytes; overflow returns no artifact. It never returns a truncated success.
Predicate composition is <=128 nodes / eight edges, equality text <=4096 bytes and
no NUL; read order <=32, limit 1..256. Existing operation-spec/IR argument, JSON,
SQL and native/result budgets also apply. Bounds constrain generation/preparation,
not arbitrary allocations made independently by the application.

## Generated API and type matrix

Each physical table gets its own module, nominal `Key`, `Model`, `Column<T>`,
`Predicate`, `Order`, `Projection<R>`, `Repository`, `Read<R>`, `Rows<R>` and a
separate private-field row struct for every configured projection. Model components
are never common aliases. Key::new requires **every** primary component in native
order, with explicit sensitive_components access. Key predicates compose balanced
trees, retaining component order within the bounded depth. Only a fully readable,
non-tenant key exposes predicate(); a key containing tenant has no public predicate
conversion. No public column factory accepts tenant values, even equal values.
Keys remain data and never grant a single-row claim by themselves.

| Native | Rust result/column | PostgreSQL | MySQL |
| --- | --- | --- | --- |
| i8 | i8 | refuse | exact signed tinyint |
| i16/i32/i64 | same width/sign | native INT2/INT4/INT8 | corresponding signed width |
| u8/u16/u32/u64 | same width/sign | refuse | corresponding unsigned width |
| bool | bool | existing native bool | existing finite bool subset |
| text | String | existing text subset | existing character subset |
| nullable supported T | Option<T> | exact native T or typed NULL | same |
| Unknown/unsupported families | no Rust fallback | refuse | refuse |

Column::eq accepts exactly T (Option<T> on nullable columns); None equality refuses
with fixed NullComparison, never rewrites SQL. Nullable Column::is_null is explicit.
Predicate::and/or preserve order and tree semantics. Order is explicit asc/desc.
PR1 deliberately exposes only this finite operator subset, not arbitrary SQL,
callbacks, function expressions, column strings, driver getters or dynamic types.
Unsupported operator/native families do not gain a dynamic execution path.

The app calls `Repository::bind(&current, &mapping)` with its existing complete
TypedModelContract. Bind compares exact target and mapping, complete ordered
ScopeFacts including every column/key and ordinary-table semantics, and exact
policy declarations for that model. Wrong target/model/alias/role/native/nullability,
read/coverage/keys/order or unsupported facts refuse. Additional protection
policies require corresponding source regeneration/application registration,
never an implicit relaxation. Other model declarations remain independent.
Subject/tenant values are only application context, never generated source defaults.

`repo.select(Model::compact(), Some(Key::new(part,id).predicate()?), order, limit)`
requires the explicit typed projection. There is no SELECT *, implicit all-column
projection or missing-field initialization. A CompactRow exposes only its selected
fields, in projection order; unselected fields have no accessor or fabricated value.
Generated code feeds closed finite JSON to the existing SpecSource compiler,
TypedModelContract and common Core preparation/finalization. It installs no SQL
renderer or risk/authorization gate. Compile/inspection use zero DB/provider calls.
Typed success still cannot relax High/Unknown/incomplete or supply a business permit.

Read<R> privately retains the same CompiledRead; inspect/public_view borrow it.
With consumer features forwarding runtime-tokio plus postgres/mysql, its consumed
execute_postgres/execute_mysql calls the existing owned session compiled-read API.
The consumer must declare these matching feature names when including the source;
the examples and fresh consumer harness compile all four finite combinations.
Current context/mapping must still exactly match retained content, including
same-generation changed contents, target, subject, tenant, policy/schema/settings.
Inspection and dispatch use the same sealed record, not re-rendered SQL. Read has
no Clone/serde/restore/into-CompiledRead/into-Operation/executor callback API.
No borrowed/atomic/write repository capability is added.

Existing driver native decode, unexpected NULL/width/sign refusal, 256 rows,
4096-byte text and 1MiB semantic-result payload limits apply before generated result
conversion. Generated row conversion independently matches native variants, exact
row length and typed NULL; mismatch revokes session availability via the
reduction-only quarantine_generated_decode method and returns fixed Decode. No
retry is introduced. No unselected field is decoded or initialized.

## Output and artifacts

CodeArtifact, generated key/model/column/predicate/order/projection/repository/read/
rows/results have fixed redacted Debug, no Display or serde. Explicit sensitive
Rust source/component/row/field access is for application-local handling. Rust
source is code data; it is neither an execution command nor public output.
No source, SQL/name/value/tenant, arbitrary compiler error, fingerprint or input path
is included in public diagnostics, Display/error/source or tracing. The generator
has no file I/O, exporter or CLI route and never overwrites a handwritten file.
Checked-in finite examples under fixtures/codegen-v1 are explicitly local code
artifacts. Tests write only dedicated temporary consumers, cleaned after use.

Output-only `quent_codegen_summary` v1 contains fixed state `unverified_code` and
model/projection/code-byte counts. `quent_codegen_diagnostic` v1 contains fixed
state `refused` and wire/version/budget/unsupported/naming/coverage/mapping code.
Both are Serialize-only; neither is an input/restoration format. Source parsing
errors map to fixed Wire; config/generator revision mismatches map to Version.
There is no public JSON roundtrip into a repository or current context.

## Evidence responsibilities and remaining work

QT-C01/C07/C08/C10 (read-only surface only)/C11/C12/C15/C19/C24 have finite PR1
subcases: production deterministic/bounded generation, independent handwritten
API/type/SQL/native/result oracles plus checked-in byte correspondence, fresh
production-output compile-fail consumer and compiling controls, real both-DB reads,
partial projection/typed NULL, exact inspected/dispatched SQL allocation, context/
mapping/High/PII/RequiredFilter refusal, native text-budget quarantine/no retry and
canary redaction. The prior 170 negatives/eight controls remain; the additional
consumer runs 37 diagnostic-checked negatives/six controls across four feature
combinations (including empty-policy, nominal single-i64-key and maximum-16-component-key outputs). Consumer
registry resolutions are checked against the seeded workspace lock. CI prefetches
that locked dependency set for standalone offline controls. Failures from missing
imports/crates are explicitly excluded.

Original 80 feature checks and 12 output combinations remain; 16 codegen overlays
and four codegen/output combinations are added. Both-OS and both-DB CI is retained.
Full runtime_pending stays on all 24 fixture cases. New-head commands/counts/CI,
server settings and disposable cleanup are execution-report evidence, not inherited
measurements or documentary fixture expectations.

PR2 retains Unchanged/SetNull/Set patches, exclusion of protected patch columns,
handwritten/generated comparison, regeneration workflow and no-overwrite management.
They are not implemented here. No new write/CRUD/native family, authorization,
MCP/command/migration/EXPLAIN capability is added. Physical-server authentication,
DDL races, arbitrary TLS, Windows live, OS-wide I/O deadlines and new write providers
retain their previous unsupported/unverified status. This PR does not close Issue8,
merge, release or publish.

## Local measurements for this PR

Ubuntu 24.04.5 / WSL2, rustc/cargo 1.94.0. Final sequential full workspace/all-feature
regression passed 162 tests, zero failed/ignored/filtered (153 retained + nine new).
Default passed 81. DB-free `--skip live_` passed 152; the three incidental substring
exclusions (Core context, manifest live claim and CLI detecting control) each passed
separately. Seven actual live suites are covered by the full run. Shared compiler
harness and fresh generated consumer ran sequentially; their 170+37 checked
negatives and 8+6 controls passed. Product all-feature CLI build and complete test
compilation preceded the final sequential full run.

Fmt, default/all-feature workspace build, warning-denied all-target clippy/rustdoc,
all 96 feature checks, all 16 output test/clippy combinations and normal dependency
trees passed. Core's normal tree is empty; default facade/CLI only contain Core.
Codegen alone only adds already-pinned serde/serde_json and locked transitives.
Cargo.lock, toolchain and existing pins are unchanged. Native Ubuntu/Windows and
new-head hosted CI results are recorded separately in the execution/PR report.

Final new disposable PG 16.14: UTC/read committed/UTF8/en_US.utf8; MySQL 8.4.8:
+00:00/REPEATABLE-READ/utf8mb4/utf8mb4_bin, existing strict modes and FOUND_ROWS.
Exact image digests remain in scripts/test-dispatch.sh. The generated fixtures use
quent_dispatch_test.generated_items under quent_test; existing catalog/CLI and
hidden-role fixtures retain their isolated namespaces/settings. Final containers
and anonymous volumes were removed and absence verified. No existing DB was used.

Development-only failures were corrected: emitter format braces and temporary
borrow; a test context incorrectly supplying two distinct tenant columns; a fixed
diagnostic field-count expectation; generated is_null ownership/clippy convention;
and formatting after test additions. Sandbox DNS, Docker socket and loopback
restrictions were distinguished from host/auth failures and resolved through scoped
approved diagnostics/execution; Docker availability initially required the answered
controller environment question. No host configuration was changed by this worker.
No final failed/ignored test is counted as acceptance. Physical authentication,
DDL concurrency, TLS, Windows live, OS I/O deadlines and write-provider guarantees
remain unchanged and unverified.
