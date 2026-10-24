# QT-AI-01 PR1: DB-free plan IR v1

Implemented in `quent-core`, with no dependencies, runtime, driver, provider,
clock, catalog, EXPLAIN, global state or execution API. Draft → owned immutable
Operation → finalize → one private Record containing final SQL, ordered native
typed arguments, dialect and semantic metadata. Finalize may be repeated on the
same immutable operation; each resulting plan is a separate immutable snapshot.
No public serde/JSON input, execution handle, policy, risk, row/scope proof,
trusted tenant, provider or dispatcher is implemented. Risk remains unevaluated;
plan success never grants permission. A column named id establishes no key fact.

## Operation and dialect capabilities

| Feature | PostgreSQL | MySQL |
| --- | --- | --- |
| SELECT | ordered explicit columns/result aliases, filter, order, bound limit | same |
| COUNT | COUNT(*) over matching rows | same |
| Single INSERT | explicit ordered column/value pairs | same |
| UPDATE / DELETE | optional finite filter, no order/limit/alias | same |
| Identifiers | double quotes, embedded quotes doubled | backticks, embedded backticks doubled |
| Arguments | $1, $2… | ? in occurrence order |
| Paths | one table or two components (schema/table) | one table or two components (database/table) |
| Table alias | SELECT/COUNT only; column references resolve to alias | same |
| Raw, arbitrary SQL/function/type string, subquery, join, CTE, upsert, batch, RETURNING, wildcard SELECT | unsupported, no input variant/API | same |
| Write order/limit/alias | Unsupported error | Unsupported error (deliberately common subset) |

`Draft::count()` applies only to SELECT and replaces its final projection with
COUNT(*), clearing order and limit, preserving table/filter. It counts all matching
rows, not rows of a limited page. The transformed operation, final projection and
ordered predicate are captured with the SQL, never reinterpreted from a public view.
Write lists reject empty/duplicate columns; omission is absence from the list,
not a supplied Missing or NULL. Missing supplied values are refused. DEFAULT values,
generated/readonly/tenant-column enforcement require subsequent schema/policy APIs.
Unfiltered writes can be planned; they have no dispatch capability. Structural risk
blocking belongs to PR2. Duplicate projection output names are rejected.

## Finite value vocabulary (native IR v1, not fixture wire v1)

| Type | PostgreSQL plan | MySQL plan |
| --- | --- | --- |
| signed i8 | unsupported (no native PG width) | i8 retained |
| signed i16/i32/i64 | exact native width/sign retained | same |
| unsigned u8/u16/u32/u64 | unsupported (no native unsigned PG type) | exact native width/sign retained |
| bool | bool retained, never integer | same |
| UTF-8 text | owned UTF-8, empty distinct, NUL refused | same finite subset |
| SQL NULL | carries finite SqlType; assignment requires nullable column | same |
| Missing | not a Value; supplied write Input::Missing refused | same |
| exact decimal, float/nonfinite, bytes, UUID, date/time, unknown/opaque | unsupported, no automatic conversion/API | same |

No arbitrary Display/Serialize/callback/SQL type strings are accepted. Native Rust
constructors cannot hold an out-of-range integer. Test-source lexical conversion
uses checked native parsing; e.g. i8 `128`, u64 `18446744073709551616` are rejected.
Fixture decimal `12345678901234567890.0100` is refused, not floated. Fixture integer
lexeme `000` represents native semantic zero; source lexemes are not runtime wire
identity. The IR type domain does not exceed fixture v1's domain, but narrows it:
no exact decimal, nullable NULL has a finite native type instead of a string.
No fixture version change or new production wire format is introduced.
This is plan capability, not SQLx encoding, decoding, server type or live schema
verification. Those require QT-AI-03 and explicit real server evidence.

## Conditions and identity

AND/OR preserve child order and explicit parentheses; NOT wraps its complete child.
Eq/Ne and integer Lt/Le/Gt/Ge compare a resolved column and a same-type value.
Text/bool ordered comparison is Unsupported. IN retains argument order, including
NULL. Empty IN emits `(1 = 0)` with no args; metadata retains the original empty IN
node and resolved column, whose defined interpretation is false even for NULL.
IS NULL is separate. Eq NULL remains `column = placeholder` and is SQL UNKNOWN;
NOT UNKNOWN remains UNKNOWN, unlike IS NULL. No boolean simplification, string
collation inference or NULL-to-equality rewrite occurs. Native bool is not 0/1.

Each Column owns exact table path/alias identity, finite type and nullability.
Cross-table/alias references and inconsistent type/nullability declarations for the
same column name are refused. These are application-supplied declarations, not
live catalog or key authority. No key metadata or name-based single-row proof.
Identifiers are 1..63 ASCII bytes of alphanumeric, underscore, quote or backtick;
embedded delimiter characters are escaped. Dots must be separate path components.
Whitespace, SQL punctuation, controls and non-ASCII identifiers are refused.
No caller text is interpolated except validated and quoted identifiers. Limits
are checked nonnegative u64→i64 and bound as typed arguments, never SQL literals.

Fixed classifications: Identifier, Shape, Type, Missing, Unsupported, Budget.
No source error/cause or free text is retained. Logical groups cannot be empty.
Budgets: 16 predicate edges, 1024 predicate nodes, 256 children/IN elements,
256 projections/write pairs/order terms, 1024 args, 4096 bytes/text, 131072 SQL
bytes. Identity path has 1..2 components. Oversize input is rejected; no truncation
or partial plan. These bound finalize work, not the caller's allocation of Draft.

## Output and evidence boundary

All sensitive carriers have redacted Debug, no Display/serde; errors expose only
fixed classifications and no source. Explicit `Plan::sensitive_source()` borrows
SQL/args/metadata for local sensitive inspection. It is not a public view, export,
permit, executable handle or dispatcher. Do not put its getter results into normal
CLI/MCP/log output. No filesystem artifact exporter is added. QT-AI-04 owns the full
public view. CLI continues its fixed unimplemented failure with empty stdout.

Tests cover 5 families × 2 dialects with SQL/args/metadata assertions, condition
ordering/NULL/empty IN, quote escaping/path/alias resolution, native widths/sign,
bool/zero/empty/missing/null, fixed refusal, budgets, immutable snapshots and canary
Debug/error boundaries. Compile-fail tests reject execution from plan/source and
unsupported float/overflow constructors. Existing QT-C01 and QT-C07 fixture sources
feed the production planner for both dialects, compared to independent SQL/args
expectations. The existing test-only recorder remains empty: no dispatcher or
provider exists to invoke. This supports the dependency-free design; it does not
measure an absent adapter, prove live DB results, or simulate policy acceptance.
QT-C02–05 proof/risk and all 24 full runtime acceptance cases remain pending.
No real DB server/image/settings, encode/decode, transaction, Windows ACL or GUI
behavior is claimed. CI runs the same DB-free tests on Ubuntu and Windows.

No production/dev dependency, lockfile, MSRV (Rust 1.94), edition (2024), default
feature or publish=false changes are required.

## Local verification

Ubuntu 24.04.5 / WSL2, rustc/cargo 1.94.0. `cargo fmt --all -- --check`,
`cargo build --workspace --locked --offline`, `cargo test --workspace --locked --offline`,
`cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings`,
`RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked --offline`,
and each `cargo check -p quent --no-default-features --features
{postgres|mysql},runtime-tokio,tls-rustls --locked --offline` passed.
30 tests passed, 0 failed/ignored: 13 existing support tests, 11 new core integration,
2 new fixture integration and 4 new compile-fail doc tests (17 added).
Core dependency tree is empty; default normal workspace tree contains only local crates.
Ubuntu/Windows CI results must be read from the actual PR checks, separately from
these local observations. No local Windows program or real DB was required/run.
