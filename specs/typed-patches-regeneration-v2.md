# QT-AI-07 planned PR2: inspection-only patches and pure regeneration

The `codegen` feature retains the exact pins, zero-dependency Core, default Core-only
facade/CLI, driver-free standalone codegen/manifest/operation-spec and existing
feature matrix. There is no new dependency, CLI grammar, filesystem exporter,
OperationSpec write grammar, runtime connection, provider, native family or migration.
All 24 full `runtime_pending` markers remain unchanged.

## Explicit generation revision

`generate(manifest_source_v1, config)` accepts only the matched pairs config version
1 / `quent-codegen/1` and config version 2 / `quent-codegen/2`. Config v2 has the
same required closed fields as v1; unknown fields, roles and unsupported/incomplete
source retain the existing refusal. Its artifact header is `quent_rust_code`
version 2. V1 remains available and its emitted bytes are unchanged. V1 is not
silently upgraded, and comparing v1/v2 artifacts returns Unsupported. The output-only
summary/diagnostic versions are independent of input/artifact revisions.

Canonical schema, policy and config domains retain PR1's exact ordered canonical
bytes. No order, target, key component, native sign/width, nullability, role or policy
is erased. Generator output is UTF-8, LF-only, final LF, without rustfmt. Both v1
and v2 checked-in `.rs` artifacts have explicit LF gitattributes. Canonicalization
of input wire whitespace remains PR1's; exact artifact byte comparison never
normalizes CRLF, whitespace or trailing bytes. Private artifact records retain the
actual revision and full canonical domains; headers and fingerprints are not inputs
to revision/correspondence decisions. No hash is needed or accepted, so a claimed
same hash cannot hide different content.

## Model-specific patch API

Every v2 model module owns a private-field `Patch` and `CompiledPatch`. Patch is
owned, non-Clone, non-Copy, non-serde, with redacted Debug and no Display. `new()`
and Default mean every column is Unchanged. Generated `set_field(T)` records Set(T),
where T is the exact native non-Option Rust type. Nullable columns additionally
have `set_null_field()`, which records SetNull separately. There is no Set(None),
nullable wrapper conversion, arbitrary column/value constructor, field erasure or
public deserialization. Model-specific Patch and Key cannot be mixed. Signed and
unsigned integer widths are checked by Rust types; no floating/string/Any fallback.
Text is owned UTF-8, <=4096 bytes and no NUL. Duplicate setters, including Set then
SetNull or SetNull then Set, return Duplicate; setters consume their receiver and
return no partially valid patch on error. There is no reset/last-wins behavior.
No changes (including models with no eligible columns) returns Empty at compile,
without inventing UPDATE defaults or a no-op statement.

Eligibility requires readable Ordinary config columns and excludes primary-key,
tenant, readonly, generated and PII policy columns. Config/read is a reduction only;
it cannot authorize writes. Unknown role, native, coverage/default/table semantics
are refused by the generator. Bind compares the full independent current
TypedModelContract, exact target, complete ordered schema/key facts and all policy
rules, including excluded fields. It therefore refuses when application mapping or
policy disagrees with generation. JSON self-asserted roles cannot override current
application facts. Existing common preparation independently rejects tenant,
readonly/generated and PII writes even when manually constructing a Core Draft.

```rust,ignore
let repo = generated_items::Repository::bind(&current, &mapping)?;
let patch = generated_items::Patch::new()
    .set_big(i64::MAX)?
    .set_null_flag()?;
let compiled = repo.compile_patch(generated_items::Key::new(part, id), patch)?;
let inspection = compiled.inspect();
// Explicit local sensitive inspection only:
let sql_and_args = inspection.sensitive_source();
let summary = compiled.public_view().to_json();
```

`compile_patch` accepts a complete model-specific primary Key in its generated
component order; no optional/general predicate or missing-WHERE route exists.
SET order is schema column order, independent of setter invocation order. Unchanged
fields never enter SET/args. SetNull creates exactly `Value::Null(SqlType::T)` and
Set(T) exactly the corresponding native variant. No expression/default is synthesized.
The patch retains one assignment per selected field. Existing common IR ownership
and final-record retention are reused, without a second renderer or binder.

Aggregate patch payload <=65,536 semantic bytes is checked during setters and
compile; final prepared ordered arguments (including key and context guards) are
also checked against 65,536. Text contributes its UTF-8 bytes; other finite values,
including typed NULL and bool, conservatively contribute eight bytes. Existing
Core identifier/IR/SQL/argument budgets also apply. Application allocations made
before calling these APIs are outside this budget.

Compilation constructs an existing Core UPDATE Draft, always with the complete
ordered key predicate, and calls common current-context preparation once. Existing
tenant context supplies the outer AND guard; patch cannot supply tenant values.
The exact resulting preparation is retained privately for inspection. It additionally
requires Low risk and Complete analysis; no partial/High/unknown promotion occurs.
`matches_current` compares full retained context/mapping contents, including changes
within the same generation, and is an inspection helper only.

## Execution boundary

CompiledPatch exposes inspection, output-only summary and exact comparison only.
It has no execute, direct Operation/preparation getter, callback, validation/dispatch handle,
serde restoration or conversion into an existing executable artifact. Compilation,
generation and comparison call no DB/provider/clock/catalog/network/environment API.
Consequently new patch dispatch/send counts are structurally zero, and new live
patch UPDATE/expected-row/expiry/session/atomic observations are not applicable.
The original Strict dispatcher remains responsible for actual ordinary CRUD, owned
validation and atomic execution, exact session/content/deadline checks, rollback,
borrowed non-atomic results, quarantine and no retry. These contracts and their live
regression suites are unchanged. Passing type checks or inspection grants neither
permission nor exactly-once behavior. MCP and OperationSpec remain read-only.

## Pure regeneration and non-overwrite workflow

Generate the exact previous production `CodeArtifact` from its recorded input and
version, then generate the proposed artifact separately. Call
`compare(&previous, &next, existing_bytes)` with optional borrowed existing bytes.
CodeArtifact's private fields prevent a marker or public JSON from fabricating the
expected prior code. The comparator does not assert ownership of an on-disk file.

| Condition (in precedence order) | State | Finite action |
| --- | --- | --- |
| Prior/proposed generator/artifact revision differs | Unsupported | regenerate_explicit_version |
| Supplied existing bytes >524,288 or invalid UTF-8 | Unsupported | resolve_locally |
| Existing bytes differ from the **complete** previous artifact | Conflict | resolve_locally |
| Existing equals previous and proposed bytes are identical | NoChange | none |
| Missing file, or exact prior file with changed proposed code | GeneratedChanged | review_new_artifact |

Missing is distinct from an empty existing file (Conflict). Handwritten code,
modified generated code, marker spoofing, same header with changed code, CRLF bytes
and trailing bytes never qualify as exact prior code. No state writes, removes or
replaces anything. Even GeneratedChanged only instructs local review of the new
artifact through explicit `sensitive_rust_source()`. A version migration requires
explicit new-version generation and separate local review; it does not obtain an
implicit apply permission. Different target/policy/ordered key/native/nullability/
projection/config domains alter exact output; `changed_domains` reports only the
count of changed revision/schema/policy/config domains (0..4), without field names.

This API accepts slices and artifacts, **not paths or filesystem handles**. It cannot
open FIFO, traverse symlink/reparse/`..`, race an existing file or leave partial files.
There is no filesystem writer or bounded-file reader to test here. External callers
that read/write files must separately enforce bounded regular-file/FIFO-safe input,
new dedicated destination, same-open-handle checks and final create_new/O_EXCL;
these are not promises made by the pure API. Local filesystem I/O, permission/race/
cleanup tests and OS I/O deadline guarantees are therefore not added. Pure comparison
and generation are portable; Ubuntu/Windows CI tests the same strict bytes. No local
Windows live DB or Windows export support is introduced. Existing SensitiveDetails
export is a separate contract: Windows UnsupportedOs, CI/MCP refusal and Linux
protection remain unchanged.

## Public outputs and evidence

`quent_patch_summary` v1 has fixed state inspection_only and assignment count;
`quent_patch_diagnostic` v1 has refused and one of empty/duplicate/native/budget/
binding/policy/gate. `quent_regeneration_summary` v1 has finite state/action and a
changed-domain count. All are Serialize-only and cannot restore mappings, patches,
context, permissions or execution handles. No SQL, name, key, value, path, source,
code/diff, default, hash or arbitrary error/cause appears in default Debug/Display/
Error/source/JSON/tracing. Code is accessible only through the existing explicit
local sensitive getter. This is separate from SensitiveDetails export opt-in.

Finite QT-C10 evidence plus C01/C07/C08/C11/C12/C15/C19/C24 is in
`tests/patch_regeneration.rs` and the extended fresh production generator consumer.
The original 37 negatives and six compiling control invocations are retained; 39 additional
negative cases verify patch/key mixing, nullable ambiguity, nonnullable NULL,
protected setters, native sign, private fields, serde/Display/type-erasure and absent
execution conversions. Fresh consumer now runs 76 diagnostic-checked negatives and eight positive controls
in six control invocations. Runtime controls check the exact cumulative
payload bound and maximum complete key. Independent handwritten SQL/ordered native
args/metadata oracles cover both dialects, all widths/extrema, typed NULL, empty text,
false, omitted assignments, duplicate/empty/native refusals and common protected
writes. Exact old/new artifacts, domain changes, modified/handwritten/marker bytes,
missing/empty/huge/invalid UTF-8/CRLF and source-free canaries exercise regeneration.
Existing FIFO three-entrypoint, read/Strict/CRUD/owned/atomic/catalog/CLI, 170-negative/
eight-control shared compiler harness and all feature/output checks remain mandatory.
C02–06/C13/C14/C16 retain their existing dispatcher/atomic measured scope; this PR
adds no write connection or full-case completion.

Physical server authentication, arbitrary concurrent DDL, TLS, Windows live, OS I/O
deadlines and a new write provider remain unsupported/unverified. Exact new-head
commands/results, hosted CI and disposable DB cleanup belong to the execution report.

## Local verification for this revision

Ubuntu 24.04.5 / WSL2, rustc/cargo 1.94.0. Final sequential full workspace/all-feature
run on new digest-pinned PG 16.14 and MySQL 8.4.8 passed 167 tests (162 retained + five
new), zero failed/ignored/filtered. The shared compiler and fresh consumer harnesses,
product CLI/catalog and generated read live suites ran sequentially. Owned API
retains 170 checked negatives/eight controls; fresh generated consumers pass 76 checked
negatives (37 retained +39)/eight controls in six invocations, four feature combinations.
Default tests passed 81; all-feature DB-free `--skip live_` passed 157 (152 retained +
five), and the three unrelated substring exclusions each passed separately. The CLI
loopback detecting control required approved execution after sandbox listener denial;
the full approved live run also includes it. No denied sandbox attempt is counted
as a successful test or host failure.

Fmt, workspace default/all-feature builds, warning-denied all-target all-feature
Clippy and rustdoc, 96 feature checks and all 16 output test/Clippy combinations
passed. Core normal tree is empty; default facade/CLI contain only Core. Codegen,
manifest and operation-spec alone remain driver/runtime-free. Pins, Cargo.lock,
workspace manifests, toolchain and CI jobs are unchanged. Development test/API/
fixture mismatches and two Clippy issues were corrected before the successful final
checks. Hosted new-head results are separate PR/report evidence.

PG measured UTC/read committed/UTF8/en_US.utf8. MySQL measured +00:00/REPEATABLE-READ/
utf8mb4/utf8mb4_bin with the retained strict modes and pinned FOUND_ROWS adapter.
Exact image digests remain in scripts/test-dispatch.sh. Existing dedicated namespaces,
roles/settings are confined to those new containers; no existing DB is used. Final
containers and their anonymous volumes were removed and absence verified. No new
patch write connection or apply filesystem is introduced, so no new patch live-write
or exporter race/permission/cleanup guarantee is claimed. Local Windows live,
physical authentication, general DDL races, TLS and OS deadlines remain unverified.
