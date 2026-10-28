Current QT-AI-13 PR1: [finite offline read-only MCP stdio](mcp-v1.md). Earlier PR-specific MCP absence statements describe their historical baseline.

Current QT-AI-05 PR2: [explicit finite catalog/CLI verification](manifest-verify-v1.md).
Earlier PR-specific absence statements below describe their historical baseline.

# QT-AI-05 planned PR1: DB-free manifest source v1

`quent/manifest` is an optional, driver/runtime-free feature. `ManifestSource::parse`
accepts a bounded sensitive source, not trusted application facts, an authenticated
context, generated repository, execution input or a verified catalog. Core remains
without dependencies. No live reader, drift SELECT, CLI workflow, exporter,
OperationSpec, MCP, code generator or permit provider is introduced. CLI retains
`QUENT_NOT_IMPLEMENTED`. Parsing, canonicalization, inspection and comparison use
no DB/provider/clock/filesystem/network calls. No manifest operation dispatches or
begins a transaction; an absent recorder is not a measured rollback/refusal.

## Source wire contract

The examples in [fixtures/manifest-v1](../fixtures/manifest-v1) are production-parser
inputs, independent of the older test-only contract fixture reader. UTF-8 JSON,
exactly one document. Envelope kind `quent_manifest_source`, version integer `1`.
`schema`, `policy`, `generated_code`, `live_observation` are optional nullable
artifact slots; absence and null both mean missing artifact. This equivalence
applies only to artifact slots, never SQL defaults or column nullability.
Every present artifact has its own required kind/version/target/coverage:

| Slot | kind | version | Finite payload |
| --- | --- | --- | --- |
| schema | quent_schema | 1 | ordered tables, columns, defaults, full ordered keys, per-table and per-key coverage |
| policy | quent_policy | 1 | ordered finite declarations, exact table path/column |
| generated_code | quent_generated_code | 1 | claimed generator version and bounded Rust source text, never compiled or verified |
| live_observation | quent_live_observation | 1 | claimed observation/expiry/generation and optional schema, always unverified |

Targets have required database/routing address/catalog/dialect. They are sensitive
source identities, not DSNs or credentials. Dialects are `postgres`/`mysql`.
Database/address are nonempty <=128 UTF-8 bytes; catalog/table/column names use the
existing Core Ident grammar (1..63 ASCII alphanumeric/underscore/quote/backtick).
Paths have 1..2 components. No case folding, Unicode normalization, path resolution,
alias synthesis or namespace inference occurs. This source vocabulary does not
expand preparation's stricter canonical two-component identity rules.

Coverage claims: `unknown`, `partial`, `complete`. Schema effective coverage never
promotes an unknown column/null/default/constraint or unsupported semantics to
Complete. Complete only describes the finite *source*, never authenticated facts
or a complete live catalog. Tables carry `semantics` (`ordinary`, `unknown`, `view`,
`triggers`, `rls`, `generated`, `collation`, `unsupported`), `columns_coverage`,
`keys_coverage`, `columns` and `keys`. Unknown/unsupported retain distinct tokens.
Effects cannot be described as supported ordinary tables by normalization.

Column `native` is i8/i16/i32/i64/u8/u16/u32/u64/bool/text/unknown/unsupported;
`nullability` is unknown/nullable/nonnull. All are required. Unsupported native SQL
types have the explicit unsupported token; arbitrary type strings are refused,
not stored in Any/metadata. PostgreSQL i8/unsigned are Unsupported coverage.
Keys retain primary/unique kind, nonempty complete ordered column names and
coverage. Unknown key coverage is not a complete key; nullable/unknown-nullability
key components produce Unsupported coverage. Non-signed key components (including
text/bool/unsigned) also remain Unsupported under the existing finite proof domain. Neither an id name nor a unique label
proves single-row semantics. This PR provides **no** conversion to ScopeFacts,
SchemaSnapshot, Column, ApplicationContext or session handles, even for Complete.
Existing explicit application assertions and Strict gates remain authoritative.

Defaults are deliberately separate:

| JSON column default | Meaning |
| --- | --- |
| field omitted | not observed |
| `{"state":"unknown"}` | explicitly unknown |
| `{"state":"none"}` | observed definition has no default |
| `{"state":"literal","value":{"type":"null"}}` | explicit SQL NULL default |
| `{"state":"literal","value":{"type":"i64","value":0}}` | exact signed native literal |
| `{"state":"unsupported_expression","expression":"..."}` | sensitive opaque expression, Unsupported; never parsed/evaluated/executed |
| JSON `null` | refused, not omission or SQL NULL |

Literal types match native type exactly (Null uses its enclosing native type).
Integer conversion is checked for each width/sign; no float/exponent fallback.
Text/bool/empty text/zero/false remain distinct. SQL NULL default is descriptive,
not a claim that INSERT is supported or a nonnull column accepts a NULL write.
Defaults are never used to synthesize SQL or execution values.

Policy rule kinds are tenant/soft_delete/pii/required_filter/readonly/generated,
with required table path and column. These are declarations in a source, not
PolicySet authentication/enforcement. No tenant value, subject, reason, fresh,
allow, generated_verified, arbitrary metadata or claimed fingerprint field exists.
Unknown fields, including these claims, refuse at every nesting level.
Generated source coverage is a source claim; code correspondence stays Unverified.
Live observation times/generation use `{"state":"unknown"}` or
`{"state":"claimed","value":<u64>}`. No current time, generation or expiry is
invented. Known expiry must be later than known observation. A nested claimed
schema must have the exact live-record target. Other independent artifact targets
can differ; differences stay in canonical content and are not reconciled into a
trusted combined context.

Parser budgets before typed allocation: <=65,536 input bytes, depth <=16 edges,
8,192 value nodes, <=1,024 array entries, <=32 keys/object, <=128 bytes/key,
<=4,096 decoded UTF-8 bytes/string. Objects detect duplicate *decoded* keys before
insertion, including escaped spellings and nested objects; no Value/map first pass
erases duplicates. Global byte bound also bounds allocations for decoded strings
and the second typed pass. Typed limits: <=16 tables/schema, <=256 columns/table,
<=16 keys/table, <=16 components/key, <=256 rules. Duplicate tables/columns/key
components, dangling key names and native/default type mismatch refuse. Unknown
kind/version, required-field omission, JSON type mismatch, invalid UTF-8, multiple
documents, malformed/deep/oversized input and number overflow refuse. No truncation,
legacy receiver or safe default verdict. Errors expose only ManifestError fixed
codes Budget/Wire/Version/Shape/Type; preflight nested budget failures use Wire.
No source cause, location, input snippet or arbitrary serde text escapes.

## Canonical and fingerprint contract

Canonical specification `quent-canonical-json/v1`: serialize only the closed typed
model as compact UTF-8 JSON, no BOM/newline/whitespace, struct fields in declaration
order (see wire.rs and frozen fixture expectations). serde_json =1.0.154 provides
JSON escaping and exact integer emission; no maps or floating point are present.
Enum tags use exact lowercase snake case. Strings preserve their decoded bytes;
equivalent JSON escapes/object ordering/whitespace normalize. Array order NEVER
normalizes: table/column/rule/key/component order is retained. Missing default stays
omitted; explicit None/Unknown/Null/expression remain different. No sorting,
coercion, default expansion, unknown-to-known mapping or unsupported equivalence.

Each artifact's canonical bytes include its independent kind/version, exact target,
coverage and entire finite payload. Fingerprint `fnv1a64/v1`: offset
`cbf29ce484222325`, for each byte XOR then wrapping multiply by `100000001b3`,
64-bit arithmetic, lowercase 16-digit hex. It is a deterministic **noncryptographic
hint**, not a signature, authentication, collision resistance or exact identity.
Collisions are possible and adversarially constructible. It is not suitable as a
security cache key or a permit. The API returns it only through sensitive access,
with redacted Debug. Input cannot supply it. `compare_offline` compares full
canonical bytes (including kind/version/target/coverage), never hash equality or
generation labels. Algorithm/canonical revision requires a new documented version;
no cross-algorithm equality is promised. Source version and each domain version
can evolve independently; unsupported versions currently refuse.

## Evidence and public boundary

| State | How produced in PR1 | Authority |
| --- | --- | --- |
| Missing | absent artifact | none |
| Unverified | any parsed artifact, including live/code claims | none |
| OfflineConsistent | full canonical equality of present domain sources | offline correspondence only; coverage may still be Unknown/Unsupported |
| ApplicationAsserted | explicit application_asserted_summary call | display label only; cannot construct trusted context |
| Stale | explicit application expiry and explicit now, now >= expiry | expired assertion; never live evidence |

MissingLeft/MissingRight/MissingBoth/Different/ExactContent are separate offline
comparison results. Unknown time/expiry is not populated from a clock or from
external live labels. Application assertion without known times asserts no expiry
check. External observation times never invoke a freshness clock automatically.
There is no LiveVerified/GeneratedVerified state, public constructor or Deserialize.
No parsed live record, even matching an offline schema with future claimed expiry,
becomes an actual LiveObserved event. Actual observation/provenance, current target,
expiry and re-observation belong to PR2's explicit live catalog I/O. Runtime freshness
policy, physical database authentication and external DDL races are not solved here.

`quent_manifest_summary` version 1 is a separate output-only kind. It contains
only fixed kind/version/coverage/freshness/support code plus bounded item counts (tables, rules,
one code source, optional live schema). It has no target/name/path/SQL/default/
value/expression/generation/time/fingerprint/source error or arbitrary free text.
Debug/Display are finite summaries; sensitive source/carrier/fingerprint Debug is
redacted; source has no Display or public serde. Summary JSON cannot parse as a
source, nor restore trusted facts/context/validated/atomic authority. No new tracing
emission or local exporter is installed; closed existing tracing/exporter APIs
refuse source carriers. Linux-only protected local-details and Windows/other-OS
fail-closed behavior remain unchanged.

## Evidence and remaining work

Production parser/API tests cover QT-C19/C24 and limited C01/C07/C08/C12/C15 on both
dialects: independent literal canonical/hash oracles, domain-only changes, content/
target/native sign-width/default/null/key order/coverage differences, unknowns,
missing/unverified/offline/asserted/stale and fake live/code claims, bounded refusal,
roundtrip escaping and canary redaction. Diagnostic-checked real API compilation
controls and negative conversions preserve the existing handle/guard/atomic boundary.
These are DB-free source guarantees, not new measured DB semantics. Existing real
PostgreSQL/MySQL dispatch/atomic tests remain regression requirements. Full/DB-free
compile harness runs are sequential; no parallel reliability claim. All 24 full
fixture runtime_pending markers remain unchanged. PR2 owns actual bounded catalog
readers, explicit live comparison, observed provenance/expiry, drift reporting and
CLI verify. Generated repository correspondence belongs to codegen's owning Issue.

The fixed support code register is Missing, UnverifiedSource, UnknownFacts,
PartialFacts, UnsupportedSemantics, UnsupportedNative, UnsupportedDefault and
UnsupportedKey. These codes describe finite source support, never arbitrary source
errors, business decisions or catalog authentication.

## Local verification (WSL2 Ubuntu 24.04.5 / Rust 1.94.0)

Final sequential all-feature DB-free run: 122 passed, 0 failed/ignored; four
`--skip live_` exclusions (two engine suites, existing Core context test and the
new `public_canaries_and_live_claim_nonpromotion` test). Both accidentally matching
DB-free tests were run individually and passed; full live run includes all four.
Full workspace/all-feature regression on digest-pinned disposable PostgreSQL
16.14 (UTC/read committed/UTF8/en_US.utf8) and MySQL 8.4.8 (+00:00/REPEATABLE-READ/
utf8mb4/utf8mb4_bin, existing strict modes/FOUND_ROWS): 126 passed, no exclusions,
failures or ignores. Dedicated containers and anonymous volumes removed and absence
verified. This preserves 115 previous tests and adds one library + ten integration
tests. The real compilation harness preserves 71 negatives + 4 controls and adds
29 negatives + 1 control, totaling 100 + 5 inside one Rust test. Default regression
remains 81 passed. These are local results, not hosted Windows or live catalog evidence.

fmt check, default/all-feature workspace build, warning-denied all-target/all-feature
clippy and rustdoc passed. The original 16 driver/runtime/TLS feature subsets plus
16 manifest overlays passed; all seven nonempty combinations of manifest and the
two output features passed tests/clippy. Core tree is empty; default normal facade
tree contains only Core; manifest alone adds only serde/serde_json and their locked
transitives, no driver/runtime. Exact serde=1.0.229 (derive) and serde_json=1.0.154
were already dev/lock pins; they become optional normal facade dependencies to
isolate strict wire parsing/canonical encoding. Cargo.lock is unchanged; FNV uses
no dependency. SQLx/Tokio/futures-util/tracing/rustix/toolchain/edition/publish pins
are unchanged. Original CI jobs are retained with manifest combinations added;
hosted results and exact head belong to the PR/execution report. No parallel
full/DB-free harness reliability, Windows ACL support, actual TLS or network
completion-fault guarantee is claimed. All 24 full runtime_pending markers remain.

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
