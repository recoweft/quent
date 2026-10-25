Current PR2 source support is independently opt-in: [finite Rust syntax candidates](source-review-v1.md). PR1 historical source deferrals below are superseded only for that feature; existing three-input wires stay unchanged.

# QT-AI-08 PR1: bounded offline review v1

`quent/review` is optional and enables the existing `manifest` and
`operation-spec` parsers with the same pinned serde/serde_json. Core still has no
dependencies. Default facade/CLI normal trees still contain only Core. CLI
`review` enables this library plus the existing Unix-only rustix regular-file
reader. It does not enable CLI `manifest`, a driver, a runtime, codegen, a new syn
or clap dependency. Rust 1.94.0, edition 2024, exact pins, lockfile and
publish=false are unchanged. Serde's already-locked transitive derive implementation
is not Rust source review.

This is M1's finite offline responsibility. Rust source/type inference/macro/cfg/
helper/alias analysis is PR2. SARIF/diff/baseline/suppression is PR3. There is no
natural-language/MCP/write-spec/provider/permit/command/migration/EXPLAIN/network
fetch/repository discovery/publishing feature here.

## Independent external plan data

Input is one UTF-8 JSON document, kind `quent_review_plan_source`, integer version
`1`. [A valid SELECT](../fixtures/review-v1/select.json) and its
[independent public expectation](../fixtures/review-v1/select.expected.json) are
checked by library and real product subprocess tests. This is a **new data wire**,
not a deserializer for an application Plan. The [documentary input schema](review-plan-source-v1.schema.json) describes shapes; only the production parser enforces decoded-byte/token budgets, duplicates and Core semantics. Existing `quent_local_source` remains
sensitive output-only; that kind, PlanView, DiagnosticView and review output are
refused as plan inputs. Export format compatibility/restoration is not added.

Every object below is closed. Required fields are listed; optional fields are
explicitly marked. JSON null is allowed only in a typed literal's `value`, never
as omission or as an unverified-object substitute. No duplicate decoded keys,
including escape-equivalent keys, multiple documents, floats/exponents, or
integers outside native i64/u64 token range are accepted.

| Object | Required fields / meaning | Optional fields |
| --- | --- | --- |
| root | `kind`, `version`, `dialect` (`postgres`/`mysql`), `operation` | `claims` |
| operation | `family`, `table`, `columns`, `projection`, `writes`, `order` | `filter`, `limit` |
| table | array of 1..2 explicit identifier components; no alias | none |
| column | `name`, `type`, `nullable` boolean | none |
| projection | ordered array of column indices into `columns` | none |
| write | `column` index, `value` typed literal | none |
| order entry | `column` index, `direction` (`asc`/`desc`) | none |
| literal | `type`, `value` (native JSON integer/bool/string or typed NULL) | none |
| and/or predicate | `op` (`and`/`or`), `args` nonempty array | none |
| not predicate | `op` (`not`), `arg` predicate | none |
| comparison | `op` (`eq`/`ne`/`lt`/`le`/`gt`/`ge`), `column` index, `value` literal | none |
| in predicate | `op` (`in`), `column` index, `values` array (empty allowed) | none |
| null predicate | `op` (`is_null`), `column` index | none |

Column indices are nonnegative native integer tokens, checked against the actual
registry length, never inferred from names. Columns have unique names. Identifiers
use the existing Core grammar: 1..63 ASCII alphanumeric/underscore/quote/backtick,
with proper Core quoting; dots are separate table components. The registry is
untrusted operation data, **not** a complete schema/key or application mapping.
There is no key/tenant/role/target/context/policy declaration input.

| Family | Required nonempty data | Other restrictions |
| --- | --- | --- |
| select | projection | writes empty; order/filter/limit supported |
| count | none | projection/writes/order empty; no limit; optional filter |
| insert | writes | projection/order empty; no filter/limit |
| update | writes | projection/order empty; optional filter; no limit |
| delete | none | projection/writes/order empty; optional filter; no limit |

Native tags are `i8/i16/i32/i64/u8/u16/u32/u64/bool/text`, preserving exact sign,
width and typed NULL. Signed/unsigned numeric ranges are checked before conversion.
PostgreSQL refuses I8 and all unsigned declarations (including unused columns);
MySQL accepts these finite native widths. Unsupported decimal/UUID/date/time/bytes/
float/opaque/native names refuse. Core independently validates all referenced
literal types, nullability, duplicate/empty writes/projections and supported
operators. Ordered text/bool comparisons and NUL text refuse through Core.
Empty IN retains Core's finite false rendering; it does not invent trusted facts.
Limit is explicit u64 checked through Core's signed argument range; zero is valid
plan data, never proof of a key or permission. Write aliases, returning, optimistic
version, batch/upsert/join/raw/function/aggregate beyond count/SQL parsing are not
in this wire. A constructible application IR may have capabilities absent here.

After streaming validation, only the private review implementation constructs a
temporary Core Draft, finalizes once with the **existing renderer**, and inspects
that record with `None` facts. No SQL is reparsed or separately rendered. No Core
carrier gets Deserialize, a public conversion or an executable return path.
Temporary objects are dropped; only finite public review data survives.

### Claims are separate from operation data

`claims`, when present, is an object with zero or more of these keys:

| Key | Accepted form (all unverified) |
| --- | --- |
| sql | bounded string; never parsed as SQL |
| args | 0..64 typed literals with the literal syntax/range checks above; no operation correspondence check |
| metadata | exactly `{family,argument_count}`; family one of the five above, count 0..1024 |
| verdict | accepted/refused/unknown |
| risk | low/medium/high/destructive/blocked/unknown |
| coverage | complete/partial/unknown/unsupported |
| precision | complete/partial/unknown/conditional_conservative |
| freshness | fresh/stale/unknown/live_verified/application_asserted/offline_consistent |
| fingerprint | string <=128 decoded bytes, arbitrary claim, never equality/authentication evidence |

Unknown fields, including permit/allow/reason/tenant, refuse. Invalid claim types
or native ranges refuse; valid but mutually inconsistent claims remain unverified.
Claims never merge into operation data, ScopeFacts or trusted assertions. Their
contents are not echoed: output says `claims=unverified` and a count 0..9 (or
absent/0). This is **format validation**, not verification of a source verdict,
SQL/args/metadata, freshness, fingerprint, original runtime or authorization.
Changing low/complete/fresh/SQL/args/metadata while retaining operation data cannot
improve the finite analysis. Same fingerprint with different data cannot hide
MissingWhere. A claimed blocked verdict is not itself a Core structural finding;
the full Core findings for the supplied data are independently retained.

### Plan budgets

The existing OperationSpec token scanner is reused **before** constructing a
JSON Value. Max source bytes 65,536, depth 16 edges, value nodes 2,048, streaming
array 256 entries, object 32 unique decoded keys, key 128 decoded bytes, string
4,096 decoded bytes. It refuses depth before descent and duplicates before any
map can erase them. The source-byte limit bounds temporary decoded string allocation.

Plan-specific limits: columns 1..64; projection/writes 0..64; order 0..16;
AND/OR 1..32; each IN/claim args 0..64; table 1..2; claim fingerprint 128 bytes.
All budgets apply together. The token node limit bounds total predicates/literals;
there is no independent unbounded tree. Core's existing depth/node/argument/text/
SQL budgets still apply, and may refuse earlier. No truncation/partial parse.
The scanner maps structural/depth/node/string/duplicate failures to Invalid;
explicit byte/collection/Core budgets may return Budget. Both exit 2 safely.

## Other inputs and explicit application review

`review::operation_spec(bytes)` uses existing `SpecSource::parse`, unchanged closed
read-only grammar and all its budgets ([source](operation-spec-v1.md)). It reports
syntax-only and literal-range checking, **not** model resolution, native-column
compatibility, binding, tenant, policy, freshness or runtime correspondence. CLI
has no independent application mapping/current context and cannot manufacture one
from a manifest/config/JSON role. Valid unresolved symbols/tenant-like names remain
unverified; explicit tenant/role/permit fields and unsupported native forms refuse.

`review::typed_spec(owned_source, &mapping, &current)` is application-only. It calls
the existing exact-content [typed compiler](typed-operation-spec-v1.md) and projects
its retained Inspection. Same-generation different content, model/target mismatch,
unknown roles, tenant predicates and protected/read-denied mapping retain the
original refusal. There is no alternate mapping/config compiler or binder.
The output describes application assertions and exact application snapshot checks,
not live authentication. Bool/NULL/soft-delete Partial/High results remain visible.

`review::inspection(&Inspection)` projects that same finalized record, retaining the
existing complete PlanView findings, scope, evidence and assumptions, without
rendering SQL, replacing facts or inspecting a second Plan. SQL precision is
`retained_record`; native/model/policy/tenant remain unverified for an arbitrary
inspection because its origin need not be policy preparation. An app may explicitly
supply the existing typed mapping/current to the typed entry instead.

`review::manifest(bytes)` uses existing bounded `ManifestSource::parse` and the
unchanged public summary ([manifest](schema-manifest-v1.md)). Its four independent
schema/policy/generated-code/external-live domains retain their own support coverage,
counts and Missing/Unverified freshness. Complete offline source support is not
application assertion or live verification. Source-parsed, offline-consistent,
application-asserted, stale and live-verified are not synonyms: this single-source
entry only parses; it does not compare two sources, call application assertion
constructors or inspect a clock. External timestamps/live claims remain unverified,
even if apparently expired or fingerprint-equal. Existing explicit offline comparison
and `manifest verify --live` remain separate workflows. Review never calls them.
All manifest byte/depth/node/string/semantic budgets remain unchanged (65,536 /
16 / 8,192 / 4,096; streaming arrays 1,024, object keys 32; domain-specific maxima
in the manifest guide). No parsed manifest supplies Core facts or an application mapping.

All library review entries are pure/deterministic for their explicit inputs. They
have no DB/provider/dispatcher/clock/environment/filesystem/catalog/EXPLAIN/network
capability or implicit validation workflow. Library inputs are borrowed bytes or
explicit application carriers, never paths. Valid parsing is distinct from runtime
correspondence. Execution safety/business permission/CI gate success is not reported.

## Public wire, precision and exits

The [closed output schema](review-v1.schema.json) is documentary and output-only, never an authority parser. `Review` is private-field, Serialize-only independent kind `quent_review`, version
1, with no restoration/conversion/execute API. JSON and pretty contain the **same
closed output**. Pretty is indented JSON; compact is JSON plus one final CLI LF.
Fields are kind/version/source/state/code/risk/coverage/precision/action/inspection/
spec/manifest/claims/claim_count. The three nullable nested summaries are produced
only from existing **public** output serializers; external JSON is never placed in
these fields. Their existing closed registers remain unchanged. All findings are
retained, without baseline/suppression/truncation. Top-level coverage/risk do not
replace the independent nested domain/inspection results.

| Precision axis | Plan data | Spec CLI | Manifest CLI | Explicit app typed review |
| --- | --- | --- | --- | --- |
| wire | checked | checked | checked | checked |
| sql | finite_data_finalized | unverified | unverified | retained_record |
| native | finite_data_checked | literal_ranges_checked | unverified | application_asserted_checked |
| model | unverified | unverified | unverified | application_asserted_checked |
| binding | unverified | unverified | unverified | exact_application_snapshot |
| policy / tenant | unverified | unverified | unverified | application_asserted_checked |
| freshness / runtime | unverified | unverified | unverified | unverified |
| external_verdict | unverified | unverified | unverified | not_applicable |

Finite-data SQL/native precision concerns **only this supplied data**, not external
SQL/args claims, native DB schema authentication, or the original application's
Plan verdict. Missing facts keep Core's High/Unknown; MissingWhere keeps Blocked.
For arbitrary application inspection wire/external_verdict are not_applicable;
other axes remain unverified unless explicitly described above. No Complete bit
covers unverified axes. Unknown and Partial are valid limited results, distinct
from malformed input errors and unsupported features.

Fixed review codes: source_parsed, incomplete, unsupported, structural_block,
application_inspection (invalid/budget are reserved finite diagnostic classifications).
Actions: supply_application_context, review_independent_domains. Output-only error
kind `quent_review_diagnostic` v1 has refused, invalid/budget/unsupported and the
fixed action check_supported_input. Debug/Display/Error/source retain no input or
cause. Error::source is None. No SQL/name/value/reason/path/snippet/default/hash/
credential/opaque metadata/arbitrary OS or serde error appears in public output.

```
quent-cli review plan     --json|--pretty FILE
quent-cli review spec     --json|--pretty FILE
quent-cli review manifest --json|--pretty FILE
```

Exactly four positional arguments after the executable (the format token is one
choice, not a literal pipe expression). Unknown/missing/extra targets, formats or
args refuse before opening any path. No automatic stdin: `-` and empty path refuse.
Path budget is 4,096 encoded OS bytes, source budget 65,536 **actual read** bytes.
`review rust --json|--pretty FILE` is a recognized unsupported PR2 target and never
opens FILE. With feature review disabled, valid review grammar returns Unsupported
without file reads; malformed grammar remains Invalid. Existing manifest verify
grammar/exit codes, three FIFO-safe input positions and unrelated route exit 1
remain unchanged.

| Exit | Result / stdout / fixed stderr |
| --- | --- |
| 2 | invalid grammar/data or budget; empty stdout; QUENT_REVIEW_INVALID or QUENT_REVIEW_BUDGET |
| 3 | valid Unknown/Partial review; public summary stdout; empty stderr |
| 4 | unsupported feature/target/native/kind/version/data; empty stdout + QUENT_REVIEW_UNSUPPORTED, or parsed manifest Unsupported summary + empty stderr |
| 5 | Refused structural/application inspection; public summary stdout; empty stderr |
| 8 | input/output I/O failure; empty stdout for input failure; QUENT_REVIEW_IO |

There is **no review exit 0**. Top-level Unknown includes supported external data
with no trusted facts and valid syntax-only Spec; Partial/Unsupported manifest
domains are preserved. App High/Destructive/Blocked inspection is Refused; other
Incomplete remains Unknown/Partial. All app review still has unverified runtime.
Source format errors are not a valid review result. Unix panic termination remains
non-success (normally 101) with only QUENT_PANIC on stderr; it is not a review
verdict/acceptance exit. The hook discards payload/location/backtrace even when
RUST_BACKTRACE/RUST_LOG are set. An output write failure can leave partial public
bytes (never a successful verdict); stderr failure is discarded, never formatted.

## File and OS boundary

Verify and review share one bounded regular-file helper. Unix opens with
RDONLY|NONBLOCK|NOFOLLOW|CLOEXEC, checks regular-file metadata on that **opened fd**,
then reads at most max+1 bytes. A writerless FIFO never blocks at open and is
refused before read. Directory/device inputs refuse, final symlinks refuse. Windows
opens the final reparse point itself, rejects reparse/device attributes, then checks
the same opened handle is regular. Other OSes are Unsupported. Path replacement
after open cannot replace that handle; concurrent in-place writes can still produce
a bounded parse refusal or a valid captured input. Intermediate symlinks can resolve.
No filesystem walk or input modification occurs.

These are byte/structure and FIFO-open guarantees, **not OS I/O deadlines**,
filesystem snapshot/authentication, physical server authentication, Windows live DB
or remote filesystem availability. No new Windows SensitiveDetails support/ACL
promise: UnsupportedOs and opt-in/CI/MCP refusals are unchanged. Product code
contains no test panic/env bypass. Test-only panic injection uses the exact same
hook in a child test harness; actual product canary tests cover success/refusal.

## Evidence and remaining responsibilities

Primary cases QT-C01/C08/C12/C15/C19/C20/C24. Independent complete plan/spec public
expectations, five families/both dialects, native widths/NULL, structural block,
claim tampering/same-fingerprint different data, old output kind/version/duplicate
escape keys/extra/missing/null/range/budget refusals and exact application mapping
compiler regression are tested. New diagnostic-checked compile-fail tests retain
the 170 existing negatives/eight controls; add nine negatives and one compiling
Serialize/control. Fresh generated consumer's 76 negatives/eight controls remain.
No public review input can yield an executable object.

Real CLI subprocesses check parsed pretty/JSON, safe streams/exits/determinism,
bytes boundary/UTF8, no automatic stdin, grammar-before-read, writerless FIFO,
directory/device/final symlink and regular/FIFO/symlink replacement with five-second
watchdogs that kill **and reap** on timeout. Native Windows tests cover regular,
directory/device/final reparse paths and regular/reparse replacement against independent expected output. A loopback detector has an actual TCP positive
control and sees no connection for valid or rejected review inputs with an ambient
DSN; the existing explicit-live CLI detecting control remains separately required.
Tracing capture has a positive control and observes no review events. No DB/provider
entrypoint exists in the pure implementation; empty capture is not a simulated
DB transaction result. OS-dependent tests and any denied attempts are distinguished
in the execution report. No arbitrary OS timeout is inferred from the watchdog.

All **24 full runtime_pending** markers stay unchanged. New write/Tx/live validation
is nonapplicable to review; retained Strict/CRUD/validated/atomic/catalog/CLI/codegen/
patch/regeneration regressions still run on new digest-pinned PG16.14/MySQL8.4.8
fixtures with dedicated namespaces/roles/settings and container/volume absence
checks. INSERT remains unverified, High/Blocked/Unknown execution refusal, borrowed
non-atomicity, quarantine/no retry and all exact current-content gates are unchanged.

Required checks retain 96 original feature checks and 16 output combinations; add
32 review feature overlays, two standalone cross-feature checks, four facade output
combinations and both-OS review-only/review+manifest CLI jobs. Original 76 hosted
checks (38 jobs for each push/PR workflow) are retained plus 12 jobs per workflow, yielding 100 checks across the two workflows. Actual head/commands/counts/failures/skips/
controls/DB settings/cleanup/hosted status belong to the RelayWeft execution report
and PR, never inferred from these documented requirements or old-head results.
