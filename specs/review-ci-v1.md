# QT-AI-08 PR3: bounded review CI presentation v1

Optional `quent/review-ci` enables existing `review` (manifest/OperationSpec and
already locked serde), without syn, driver, runtime, new parser or lock changes.
`source-review` independently adds Rust support to this route. CLI `review-ci`
enables the same facade feature and existing regular-file reader. Core remains
zero-dependency, default facade/CLI Core-only. Rust 1.94, edition2024, publish=false
and exact pins remain unchanged. Existing four review targets, output kinds and
exits and three manifest-verify inputs remain unchanged.

## Trust and collection boundary

Library `InputSet::parse`, `Baseline::parse`, `compare` and application
`InputSet::inspections` are pure: no DB/provider/clock/environment/Git/process/FS.
All private finalized Inspection findings and original complete review output are
retained. Projection uses original FindingCode and PR2 diagnostic enums; there is
no SQL renderer, type inference, public-output reinput, mapping or execute path.
The existing common typed compiler and policy/dispatch gates are unchanged.

The new closed input wires are independent of PlanView/DiagnosticView/Review/
SARIF/manifest-summary output. They cannot recover trusted plans/context/mappings/
permission or turn external complete/fresh/success/fingerprint claims into evidence.
Context/target/scope labels are sensitive **untrusted comparison labels**, never
application authentication or target verification. The operation source is analyzed
through existing review functions. Product does not run input commands, build.rs,
proc macros, cargo, Git, network, directory discovery or repository traversal.

```
quent-cli review-ci --json|--pretty|--sarif HEAD_INPUTS BASE_INPUTS BASELINE
```

Exactly five positional arguments after executable; three explicit files required.
No flags for success, complete, now, authorization, remote upload or export.
Empty sets/baseline have explicit fixtures. No stdin `-` or empty path, unknown
mode/extra/missing args. All grammar, path lengths and disabled feature are checked
before any file open. Valid grammar with feature off: exit4. Existing regular-file
helper protects **all three inputs**: Unix NONBLOCK|NOFOLLOW|CLOEXEC, Windows opened
reparse/device refusal, metadata on same opened handle, bounded actual read.
Clock collection is separate CLI SystemTime since Unix epoch; failed/out-of-range
clock becomes unknown, disabling hiding. JSON/environment cannot supply current time.
Library callers pass explicit `Option<Time>`; this is an application time assertion,
not authenticated time. Same inputs/time give identical results.

Same-opened-handle is not FS snapshot/OS-wide I/O deadline/immutable concurrent
content/physical-server authentication. Intermediate symlinks can resolve. Windows
live DB and sensitive-export ACLs remain unsupported/unverified. No new exporter:
stdout only, no file overwrite/ACL claim or implicit GitHub upload.

## Input grammar and budgets

[Input schema](review-ci-inputs-v1.schema.json), [baseline schema](review-ci-baseline-v1.schema.json),
[output schema](review-ci-v1.schema.json), [SARIF subset schema](review-ci-sarif-v1.schema.json)
are documentary; production streaming and semantic validation govern acceptance.
Every object is closed, every field required; no null/float/exponent/duplicate
including escaped-equivalent key/multiple document. Before tree materialization,
the existing streaming scanner bounds source bytes 65,536, depth16, nodes2,048,
array256, object32 decoded unique keys, key128 bytes, decoded string4,096 bytes.
Byte ceiling is checked before parsing. Strings/UTF8/token/depth/node/duplicate
budgets are enforced during streaming before a Value or wire struct can erase them.
No arbitrary serde/OS/source error is retained or printed.

Input root: `kind="quent_review_inputs"`, integer `version=1`,
`analysis_version=1`, `selection="explicit_set"|"changed_only"`, `inputs` array0..8.
Each input: `target`, `context`, `scope` (nonblank1..128 decoded bytes, no controls
or wildcard `*`), `format="plan"|"spec"|"manifest"|"rust"`, `source` UTF8 string.
The source is at most4,096 decoded bytes due to the outer scanner, a deliberately
smaller envelope than old direct inputs. Existing per-format checks still apply,
including Rust4KiB/512 significant/48 raw punctuation/16 raw grouping envelope.
Quoted/comment bytes count. Rust without source-review returns Unsupported.
Missing/unsupported/invalid/over-budget inputs never silently drop from aggregation:
parse fails non-success and returns no partial report.

Baseline root: `kind="quent_review_baseline"`, `version=1`, `analysis_version=1`,
`entries` array0..8. Entry: `code`, `input` (the complete same input object above),
`reason`, `owner`, `expires`. Nonblank reason1..256 bytes, owner1..128 bytes, no
controls; both are sensitive, never published. Duplicate code+exact input rejects.
The baseline input itself must pass the same review parser; unsupported/malformed
source cannot enter by baseline. Unknown fields/kind/version/types/overflow reject.

Expiry format is a native JSON **integer Unix second**, range1..253402300799
(last second of year9999), no strings/date guessing/floats/negative/overflow/null.
`Time::unix_seconds` accepts0..253402300799. Hiding requires known current time,
`now < expires` and remaining lifetime at most31,536,000 seconds (365 days).
At expiry, after expiry, unknown time or remote-future expiry: no hiding. A finite
maximum and remaining-lifetime ceiling prevent effectively infinite hiding; a
baseline has no implicit expiry, wildcard, whole-repo or inherited scope mode.

## Exact diff, advisory hiding and complete-head limits

Diff is **display only**, per diagnostic with full operation multiplicity preserved:

| Classification | Required comparison |
| --- | --- |
| unchanged | Exactly one identical full input in each set, same parser/analysis version |
| added | Exactly one full head input; no exact base counterpart (conservative changed/rename handling) |
| removed | Exactly one full base input; no exact head counterpart |
| unknown | Repeated identical input on either side, or application Inspection with unavailable original context/facts |

Equality compares full source **bytes**, format, target/context/scope labels, under
this fixed analysis version. It does not use public code/count/summary/hash or a
supplied fingerprint. Reordering unique inputs does not change classification.
Renames, whitespace edits, same names/different operations and changed content are
conservatively added+removed. Exact identical duplicate operations are ambiguous,
never paired by index or deduplicated; all diagnostic occurrences survive.
Labels themselves are untrusted: unchanged means identical supplied review data,
not verified original application operation, physical target, runtime or Git head.
Application Inspection projection has no original full context/facts snapshot here;
it is always unknown for comparison and cannot be baselined. No optimistic equality
is invented from its public view. Old base evidence never upgrades head authority.

The **only suppressible code** is new informational `untrusted_claims_present`:
"External display claims were supplied." It is separate from every existing
classification and never means claims verified. It occurs only when PR1 retains
external claims. Exact whole input must match one unambiguous head record, same
analysis version, with valid reason/owner/expiry and explicit current time.
No matched notice changes claims=unverified or any precision axis. The report keeps
this diagnostic with `hidden=true`; SARIF omits this one notice. No reason/owner/
expiry/label/source/hash is included. Baseline is never an execute permit.

ALL original Core findings stay non-suppressible: missing_where, unknown_proof,
bulk_write, weak_predicate, read_without_limit, read_projection, insert_unverified.
All source diagnostics stay non-suppressible: identity_unverified,
context_unverified, no_candidates, macro_unexpanded, attribute_unevaluated,
names_unresolved, control_flow_unresolved, receiver_unresolved,
arguments_unverified, unsupported_syntax. Also non-suppressible: review_incomplete,
structural_block, unsupported, input_budget. Tenant/freshness/type/identity/runtime
failures cannot be baseline codes. No original code is renamed to bypass its
classification. Existing policy errors refuse before an Inspection is obtained;
the existing preparation/typed APIs and full runtime gate retain their refusals.
Every report includes review_incomplete for every head record. Independent full
precision, coverage, risk, findings and manifest domains remain in head_results.

**No CI pass/exit0 exists in v1.** Parsing all explicitly listed inputs establishes
only `explicit_set=processed` (empty is missing) and a count, not enumerating all
files/operations in the actual Git head/tree. Changed-only is explicit and incomplete.
`git_head=unverified`, `runtime=unverified`, `authority=not_granted` always. No external
manifest/CLI/env complete flag is supported. Original source syntax-only candidates
never become type/runtime verification. Complete offline manifest support and
external claims never fill unverified axes. Removed diagnostics, unchanged display,
hiding or empty head never produces success. Repo-wide verified collection remains
unimplemented; no test-only product success exception is provided.

Report owns ALL head/base review outputs and all original diagnostics including
hidden ones. `diagnostics()` returns full bounded list. Internal cap256, public
presentation64 including base diff and hidden entries. Above64: state=budget,
public_budget_exceeded=true, full diagnostic_count/hidden_count retained, no detail
arrays emitted, exit2. Original private collections remain available internally;
this is explicit budget refusal, not successful silent truncation. Parsing8+8
bounded inputs also bounds full report allocation. Full head inspection exit is
computed independently of base diff and hiding: Blocked/Refused5 precedes
Unsupported4, otherwise Unknown/Partial3. Public budget refusal2 overrides those
only with an explicit non-success refusal. No execution gate consumes this report.

## Public outputs, SARIF and exits

JSON/pretty kind `quent_review_ci` version1, closed fields in schema. Pretty is
indented same document. Both include all full old review/source outputs (unless
explicit public-budget refusal), closed codes/diff/hidden flags and counts.
No sensitive input string is copied into a report. Debug/Display/Error/tracing/
panic retain only fixed registers or redaction. Errors use existing fixed review
codes and no source chain; panic uses existing fixed QUENT_PANIC hook.

SARIF supports exactly version**2.1.0**, one run, `tool.driver.name="Quent"`, and
`results` with exactly `ruleId`, `level`, `message.text`. Messages are fixed
compile-time dictionary. Original risk/coverage/independent precision/counts are
available in companion JSON; SARIF deliberately makes no lossy "safe" summary.
A fixed incomplete/error result exists even for empty/removed-only/hidden output.
Each nonhidden head/unknown result uses fixed code/error (claims notice note).
Removed display diagnostics do not become head SARIF findings. Budget emits only
incomplete+input_budget errors. No arbitrary Diagnostic/serde/OS/token message.

| SARIF member | Support |
| --- | --- |
| version / runs / fixed tool / ruleId / level / fixed message.text | finite subset above |
| artifact URI/path/baseURI, physical/logical location, region/line/span, snippet | omitted, never collected for output |
| SQL/names/values/reason/owner/credentials/source/error/hash/fingerprint | forbidden |
| markdown, arbitrary help URI/properties, fixes, external property files, fingerprints, suppression metadata | omitted |

SARIF output is a display artifact, never execution/CI authority input. No URI,
markdown or control-bearing source can influence its fixed dictionary. Anonymous
SARIF intentionally cannot serve as internal exact operation identification.

| Exit | Meaning / streams |
| --- | --- |
| 2 | grammar/input/baseline/budget invalid: empty stdout + fixed review stderr; public presentation budget: finite budget report stdout, empty stderr |
| 3 | processed finite Unknown/Partial/missing/changed-only report, stdout, empty stderr |
| 4 | disabled feature/unsupported input: fixed review stderr; parsed Unsupported head: report stdout |
| 5 | original full head Refused/Blocked, report stdout, empty stderr |
| 8 | fixed QUENT_REVIEW_IO; input failure empty stdout, output failure may leave partial public bytes |

All exits are non-success. Neither syntax/CI presentation nor baseline claims DB
execution safety, external authorization or runtime acceptance.

## Evidence responsibility

QT-C01/C02/C08/C12/C15/C19/C20/C24: independent JSON/pretty/SARIF golden streams,
actual JSON field allowlist, exact-content/target/context/scope/claimed-hash diff,
order/duplicates/rename changes, expiry before/at/after/unknown/far-future,
non-suppressible register, hidden Blocked retained, changed-only/removed/missing
head refusals, bounded/deep/UTF8/escaped duplicate input, all three CLI regular-file
entrypoints, FIFO/device/directory/symlink/reparse/path-replacement and kill+reap
watchdogs. Loopback and tracing detectors retain actual positive controls.
Shared compiler retains189 negatives/10 controls and adds14/1 =>203/11; fresh
consumer76/8 across six invocations unchanged. Existing196 feature checks plus66
=>262; output combinations25+5=30; CLI matrix5+3=8 per OS. Hosted66+16=82 jobs per
event,164 for push+pull_request, evaluated on actual exact head.

No new write/Tx/live path. Existing Strict/CRUD/validated/atomic/catalog/CLI/FIFO/
codegen/patch/regen requires fresh dedicated digest-pinned PG16.14/MySQL8.4.8
regressions and verified container/anonymous-volume deletion. All24 full
runtime_pending markers remain. Actual tested commands/counts, both OS CI, failures,
approval results, remaining limits and exact head belong to implementation evidence
and the fixed RelayWeft report, not inferred from this design or PR creation.

[Local measured evidence](review-ci-v1-evidence.md) distinguishes actual executions
from unverified limits; exact-head hosted results are in the RelayWeft report.
