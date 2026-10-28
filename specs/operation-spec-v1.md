Current QT-AI-13 PR1: [finite offline read-only MCP stdio](mcp-v1.md). Earlier PR-specific MCP absence statements describe their historical baseline.

Current PR2: [typed compilation and explicit owned read](typed-operation-spec-v1.md).
PR1-specific implementation/absence statements below describe the historical baseline.

# QT-AI-06 PR1: bounded read-only OperationSpec

Optional facade feature `operation-spec` enables production `operation_spec` APIs.
Core still has zero dependencies; default facade/CLI still depend only on Core.
The feature uses existing exact serde 1.0.229 / serde_json 1.0.154 and locked
transitives. It enables no driver, runtime, CLI route, manifest parser or I/O.

## Closed wire and meaning

Source kind `quent_operation_spec`, integer version `1`, one UTF-8 JSON document:

```json
{"kind":"quent_operation_spec","version":1,"model":"items","fields":["id"],"filter":{"op":"eq","field":"id","value":{"type":"i64","value":42}},"order":[{"field":"id","direction":"asc"}],"limit":10}
```

[Source JSON Schema](operation-spec-v1.schema.json) is documentation, not the
production parser: it cannot establish duplicate-token rejection, byte budgets or
integer token spelling. Only `filter` may be omitted. Omission is no caller predicate; explicit null is
refused. Every object is closed; all listed fields within its chosen form are
required. Native JSON null is accepted only as `value` inside a typed literal.
Model/field symbols are 1..63 bytes of `[a-z0-9_]+`, never SQL identifiers, table
paths or aliases. Parsing does not resolve existence, permission, physical mapping,
column type/nullability, single-row scope, dialect encoding or policy compliance.

| Form | Fields | Accepted syntax / refusal |
| --- | --- | --- |
| root SELECT | kind/version/model/fields/filter?/order/limit | SELECT is implicit in the independent source kind; no operation switch |
| fields | array of symbols | 1..64, explicit order preserved, duplicates refused |
| order | array of `{field,direction}` | 0..16; `asc`/`desc`; repeated field refused even with same direction |
| limit | native integer token | required 1..10000; no default, clamp or scope/authorization claim |
| and/or | `{op,args}` | 1..32 children; no empty identity rewrite |
| not | `{op,arg}` | exactly one predicate, tree retained |
| eq/ne/lt/le/gt/ge | `{op,field,value}` | one typed literal; no NULL rewrite or coercion |
| in | `{op,field,values}` | 1..64 typed literals, order/duplicates/NULL retained; empty IN refused |
| is_null | `{op,field}` | no value; NOT wraps this for IS NOT NULL |
| literal | `{type,value}` | explicit native tag, no implicit casts |

Empty IN refusal is the narrower source syntax contract, not a change to Core's
existing empty-IN false semantics (QT-C07). Empty order is explicit absence of
ordering. JSON object key order is insignificant; array/predicate order is retained.
Duplicate fields/order are errors, not deduplicated. No predicate is discarded as
contradictory or dead. All paths, including NOT/OR/IN, undergo the same validation.

| Type tag | Non-NULL value syntax | Range |
| --- | --- | --- |
| i8/i16/i32/i64 | native JSON integer token | exact signed width min..max |
| u8/u16/u32/u64 | native JSON integer token | 0..exact unsigned width max |
| bool | native boolean | false/true |
| text | UTF-8 JSON string | decoded bytes bounded below |
| any supported tag + null | explicit typed SQL NULL | tag preserved; not missing |
| decimal/UUID/date/time/bytes/float/other | unsupported | refused, including typed NULL |

serde's integer token decoding uses checked i64/u64 (never f64 acceptance). Width
checks use checked native conversions. Out-of-i64/u64 tokens, fractions/exponents,
stringified numbers and bool-as-number refuse. No new Core type, driver bind or SQL
expression is introduced. Type tags remain untrusted: mismatched application column,
nullable/NOT/NULL semantics, PostgreSQL unsigned/I8 support and other dialect details
are PR2 typed compiler responsibilities, not PR1 schema verification.

Everything outside this table refuses: arithmetic/functions/aggregate/join/subquery/
union/rawSQL/tablepath/alias/free metadata/custom callbacks/write/batch/returning/
command/apply, and tenant/subject/allow/reason/fresh/permit/policy/settings/schema/
target/generation claims. The root has no trusted-ref or tenant value field.

## Resource and parser boundary

A streaming serde seed first visits all tokens without constructing Value/maps that
could erase duplicate keys. It caps recursion BEFORE descending, validates decoded
key identity (including escape-equivalent keys), then finishes the single document.
Only after success is the bounded tree materialized and closed semantic validation
performed. This is production code, independent from manifest and fixture loaders.
The manifest's kind/version does not select this parser and there is no fallback.

| Budget | Maximum |
| --- | --- |
| source bytes | 65536 (before UTF-8/JSON decoding) |
| streaming value depth | 16 edges from root |
| streaming value nodes | 2048 |
| any streaming array | 256 entries |
| any streaming object | 32 decoded unique keys |
| decoded key / string bytes | 128 / 4096 |
| fields / order / per IN | 64 / 16 / 64 |
| predicate nodes / total AND+OR child edges | 256 / 256 |
| per AND/OR | 32 children |
| total literals | 512 |
| total literal bytes | 16384 |

Literal bytes count decoded text bytes, each non-NULL integer as 8 bytes, bool as 1,
NULL as 0. Generic node/byte limits may reject earlier than semantic maxima; all
limits apply together. Even a single decoded string buffer is bounded by source
bytes before allocation. Depth failures occur before recursive traversal/clone/drop
of any unbounded tree. Native serde errors are discarded; failures expose only
closed SpecError codes, with no source chain/position/path/snippet/error text.
Budget and structural scan failures may share Wire; there is no partial success.

## Application role assertion and authority

`SpecSource::parse` returns private immutable syntax/range-checked data. Its public
state is `syntax_only`, never schema-verified or tenant-value-prohibited verified.
`ModelContract::application_asserted` is **only for authenticated, authorized server
code**, not AI/HTTP source claims. The app asserts one exact model mapping with up
to 256 unique canonical symbols, no physical column aliases, accurate column roles
(`Ordinary`, `Tenant`, explicit `Unknown`) and read restrictions. Every potentially
used column must be covered; partial coverage never means policy-free coverage.
The app must classify all tenant columns accurately. Constructor success validates
structure, not authentication, live schema, physical target or assertion accuracy.

The immutable contract retains the complete cloned DatabaseContext. `check_roles`
consumes source/contract and requires explicit current context with exact
`same_snapshot` content equality (including target, generations and retained facts,
not merely FNV/identity/generation equality). Model mismatch, missing column or
Unknown role returns Unverified. Denied read returns ReadDenied. Every occurrence
in projection/filter/order is resolved; every predicate use of a Tenant column
refuses, including nested NOT/OR, IN and IS NULL. Names such as `tenant`/`id` carry
no inferred roles. Tenant projection/order is allowed only with asserted read access;
this does not authorize PII or override existing policies. No value is accepted for
a tenant predicate, even when equal to the application's tenant.

The result `RoleCheckedSource` retains both immutable source and exact contract.
`matches_snapshot` compares retained complete context, not freshness or physical
server authentication. An app that supplies stale/false assertions is outside this
conditional boundary. Contracts are not imported from parsed manifests/public JSON/
live observations/fingerprints. No global registry, catalog, clock or ambient tenant
exists. Only application context can supply tenant to future common preparation.
Different model/target/context/subject/same-generation different content cannot be
substituted after checking. No constructor restores these carriers from public data.

## Input/output and PR boundary

Source, contract and role-checked carrier have constant redacted Debug, no Display,
Serialize/Deserialize, From/Into to operation/context/validated/atomic/observation,
or execute API. `sensitive_source()` explicitly returns a borrowed bounded JSON tree
containing names/literals; callers must not log it. This access is intentionally
outside public output traits, and never constructs a Core Operation or trusted context.

Output-only `quent_operation_spec_summary` version 1 contains fixed syntax_only or
application_roles_checked state and bounded counts only. Independent output-only
`quent_operation_spec_diagnostic` version 1 contains one closed error code. Neither
has Deserialize or public-field restoration. No source names/paths/SQL/values/tenant/
free text/fingerprint or source errors appear in public JSON/Debug/error chains.
The new APIs emit no tracing and require no provider/DB/query/Tx/catalog/EXPLAIN/
filesystem/environment/clock access. Their signatures provide no such capability;
behavior is deterministic for both dialects. CLI unsupported routes are unchanged.
Windows local-details UnsupportedOs, catalog finite versions/semantics, unverified
Windows live support and absent OS-wide I/O response deadline remain unchanged.

PR2 owns application column/type resolution into the **existing common planner**,
NULL/operator/dialect checking, typed compile diagnostics/proposals and application
read execution via the shared preparation/dispatch gates. No compiler, SQL renderer,
execution adapter, natural-language/MCP interpretation or new family is delivered here.
Limit is never key/scope proof, policy success or authorization. Existing PII and
RequiredFilter policies remain enforced by their existing preparation boundary.

## Evidence scope

`tests/operation_spec.rs` exercises real production parser APIs with independent
wire/tree/summary expectations, both context dialects, each integer width's extrema
and overflow, unsupported families, NULL/missing, UTF-8/escaping, closed operators,
duplicate/escape-equivalent keys, huge/deep/multiple documents and budgets. C01/C24
are primary; C07/C08/C11/C12/C15 cover the limited source/role/output boundary.
These fixtures are production parser inputs, not the documentary contracts-v1 loader
or completed runtime scenarios. API harness additions have a compiling control and
compiler-diagnostic-checked negative restoration/conversion tests. Canary tests cover
Debug/JSON/Display/Error/source and zero implicit tracing. Structural API absence
establishes no DB/provider/query/Tx invocation in these methods; it is not an executed
SQL/Tx refusal measurement. Existing real DB tests separately regress the dispatcher.

All 24 full `runtime_pending` markers stay unchanged. Typed compile, full case
acceptance, physical schema authentication, live Windows behavior and broader SQL
semantics are unverified. Commands/results, failures/skips and exact-head hosted CI
are recorded in this revision's report, not inferred from this specification.
