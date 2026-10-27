Current QT-AI-11 PR2: [phase-bound finite migration observations](migration-preflight-v1.md). Earlier PR-specific absence statements describe their historical baseline.

# QT-AI-11 PR1: offline immutable migration differences

`quent_core::migration` is a dependency-free typed application API. It compares
owned `MigrationSchema` declarations and finalizes one private `MigrationPlan`:
exact before/after snapshots, typed differences, final SQL sequence and its public
view. It is separate from CRUD `Plan` and cannot become an Operation, prepared
operation, context, validated handle, permit or executor. There is no migration
provider, driver callback, transaction adapter, preflight, apply, Raw, MCP tool or
CLI input added here. Plan/compile/review make **zero DB/provider calls**. The
planner has no runtime, network, clock, filesystem or provider dependency/argument.
An independent test recorder remaining empty is structural DB-free evidence, not
an observed server refusal/rollback or an instrumented provider invocation.

## Authority, identity and compatibility

`MigrationTarget::new(database, address, catalog, dialect)` names an explicit
application target, not credentials/connection options. Both schemas carry
`SchemaAuthority::{UnverifiedSource, ApplicationDeclared}` and schema enumeration
`Knowledge::{Unknown, Partial, Complete}`. Authority changes are differences and
refuse SQL. Neither authority is authenticated, live verified or authorization.
Every table has exactly namespace/table identifiers, no alias or search-path
resolution. Full exact database/address/catalog/dialect identity must match; no
hash is used. MySQL namespace must equal catalog. Case/quoted identifier bytes
are retained, never folded for matching. MySQL constructors additionally reject
case-insensitive name collisions; SQL generation restricts namespace/table/new
column to lowercase ASCII/digits/underscore because server case settings are not
observed. PostgreSQL quoting doubles embedded double quotes using the same finite
Core identifier rules. No target declaration authenticates a physical server.

Existing manifest/source v1 wire, fingerprints, canonical array order and verify-v1
semantics are **unchanged**. V1 has no named index/FK/CHECK definitions or complete
migration coverage. There is deliberately **no implicit or explicit conversion**
from `ManifestSource`, a v1 catalog observation, public JSON, or Fresh summary into
`MigrationSchema`. Callers must explicitly supply the new typed declarations;
information not observed must stay Unknown/Partial. Passing migration fields to
the existing v1 parser is still refused as unknown fields. Migration view JSON is
refused as a manifest source. In particular PostgreSQL's normalized missing
attrdef and MySQL nullable COLUMN_DEFAULT=NULL cannot distinguish no default from
explicit NULL: such observations must remain Unknown, never become Absent.
No source parser, new external input wire, general expression parser or arbitrary
SQL-type string is introduced. Unknown SQL types use `Observed::Unknown`, retaining
unknown coverage; they cannot supply an executable native type.

## Types and complete comparison matrix

`MigrationTable::new` owns columns/indexes/FKs/CHECKs with independent enumeration
coverage for each. `TableEffects::Ordinary` explicitly declares base table and no
unrepresented effects (triggers, RLS, generated/identity columns, inheritance,
partitioning, collation, etc.); Unknown/Unsupported never becomes ordinary. This is
an application declaration, not a new way to verify a live catalog.

| Area | Compared information | Unknown or unsupported handling |
| --- | --- | --- |
| Target | full database/address/catalog/dialect, authority | mismatch refuses entire sequence |
| Tables | exact namespace/table presence, retained table enumeration order, schema coverage | no rename/move inference; add/remove/order unsupported |
| Effects | explicit Ordinary/Unknown/Unsupported and changes | unknown/unsupported refuses, even if equal |
| Columns | exact name add/remove, retained common-column order and insertion position | drop/reorder/non-tail insertion unsupported; no name-based type inference |
| Types | exact Core SqlType width/sign/bool/text | unknown stays unknown; PG i8/unsigned unsupported even if equal |
| Nullability | Observed known bool versus Unknown | unknown refuses even if equal |
| Defaults | Unobserved, Unknown, Absent, exact native typed Literal (including typed NULL), Opaque | all distinct; opaque syntax never establishes semantic equivalence |
| Indexes | exact name, primary/unique/nonunique, full component order and asc/desc, enumeration order | plain_btree explicitly excludes extra attributes; unknown or opaque expression/partial/include/prefix/opclass/collation/other definitions stay unknown even if equal |
| FK | exact name, local and referenced component order, full referenced namespace/table, delete/update actions, enumeration order | finite enforced, nondeferrable MATCH SIMPLE only; other definitions Unknown; known references must resolve inside source scope |
| CHECK | exact name, opaque bytes and enumeration order | content changes detected, but CHECK semantics always unknown, including identical bytes |

Coverage changes are retained as typed differences; coverage is independently
reported even if no represented definitions differ. A schema/table with Partial
coverage cannot prove that unlisted objects are absent. Unknown definitions on
both sides never justify NoChange. For table add/remove the parent table
presence difference is recorded, with both complete snapshots retained; absent
child objects are not fabricated as separately observed changes. Column presence
is a Columns difference; Types/Nullability/Defaults still report the knowledge of
all supplied columns. Same-name definitions are compared by content, never by name
alone. Table/column/index/FK/CHECK enumeration order is preserved conservatively;
no normalization erases a difference. CHECK/opaque expression byte equality is
only byte equality, not SQL equivalence.

`ForeignKeyDefinition::simple` preserves NoAction/Restrict/Cascade/SetNull/
SetDefault tokens for comparison. MySQL SetDefault is unsupported as DDL; no FK
SQL is generated. Plain index and FK comparisons describe finite declarations,
not a validated reference-key/enforcement proof or all dialect semantics.

## SQL capability matrix (distinct from comparison coverage)

| Change | PostgreSQL | MySQL |
| --- | --- | --- |
| Append nullable signed i16/i32/i64 column, Absent default, explicitly Ordinary table, complete known coverage, no table indexes/FKs/CHECKs | Supported offline SQL | Supported offline SQL |
| Same with i8 | Unsupported | Supported as TINYINT |
| Add unsigned/bool/text, defaulted/non-null column, non-tail position, constrained/effectful table | Unsupported | Unsupported |
| Drop/type/null/default change; index/FK/CHECK add/drop/definition/order; namespace/table add/drop/reorder | Unsupported | Unsupported |
| Unknown type/default/effect/definition or incomplete enumeration anywhere | Unknown/refuse entire plan | Unknown/refuse entire plan |
| DDL transaction semantics | PostgreSQL transactional DDL for this finite subset; no Tx is opened by planner | ALTER TABLE implicit commits; no multi-step rollback promise |

Supported SQL is only `ALTER TABLE <quoted namespace>.<quoted table> ADD COLUMN
<quoted name> SMALLINT|INTEGER|BIGINT [MySQL: TINYINT] NULL`. Every name is typed;
there are no values/default expressions/raw SQL input or user-provided renderer.
Steps follow retained table order, then new-column order, which proves append
ordering for this constraint-free subset. Existing-column removal/reorder or any
other unsupported change invalidates the **whole** sequence. The final private
record contains **no SQL** for Refused plans: not even supported-prefix SQL is
exposed through `sensitive_source().sql()` or local details.

`MigrationStatus` is NoChange (complete known finite declarations with no detected
difference), SqlPlanned (complete finite offline SQL), or Refused. Risk is Low,
High or Blocked respectively. NoChange is not current DB equality or successful
application. SqlPlanned is not an execution handle, preflight or authorization.
Lock acquisition, table rewrites, concurrent DDL, unknown catalogs, application
misdeclarations, type conversion/data loss outside this subset and external effects
are not proved safe. Even nullable append may block workloads; all actual SQL
plans are High. No database connection or side effect occurs in product planning.
PR2 owns live target/plan/phase/conditions/time/expiry/visibility-bound preflight and
expand/backfill/contract; PR3 owns explicitly controlled human application. No
universal DDL, rollback, zero-lock, physical-identity or race guarantee is made.

## Bounds and output boundary

Constructors reject invalid shapes/type mismatches, duplicate names/components,
dangling known references and budgets; they do not truncate. Limits: 16 tables,
256 columns/table, 16 indexes/FKs/CHECKs each per table, 16 unique ordered components
per index/FK side, Core identifiers 1..63 bytes, target database/address 1..128 bytes
(no controls), opaque/text default 4096 bytes. Complete column enumeration must
have at least one column. Native values cannot overflow their Rust width; literal
native types must agree when known. There is no recursion or public external-input
allocation. Work is bounded by these dimensions; the maximum supported append
sequence is 16*255=4080 steps (an existing table needs a column). Unknown/unsupported
at the final table still clears every SQL step. Caller allocations before typed
constructors are outside the retained-input budget.

All sensitive types, including generic Observed, defaults and opaque definitions,
have redacted Debug, no Display/serde/error cause. `MigrationView` is output-only:
fixed kind/version/dialect/status/risk/DDL transaction/authority/live plus bounded
steps and ten fixed areas, each with change count/unknown/unsupported. Debug and
Display expose only this allowlist. [Separate JSON schema](migration-plan-v1.schema.json)
describes the output, without changing existing public-view schemas. No type,
identifier, default, expression, SQL, arbitrary error or fingerprint appears.
There is no tracing/CLI/SARIF/MCP emitter accepting sensitive migration objects.
No public-view parser or Deserialize exists: unknown fields/versions/duplicates/
multiple documents cannot be used to restore a plan because **all** input JSON is
unsupported at this boundary. Existing bounded input parsers retain their tests.

Explicit `local_details::create_new_migration` (existing `local-details` feature)
uses the same LocalOptIn, LocalApplication surface, finite environment refusal,
protected Linux descriptor traversal and 0600 exclusive new-file/no-overwrite
backend. CI/MCP/Remote refuse; Windows/non-Linux return UnsupportedOs before file
action (after the shared CI/surface guard). The separate output-only artifact has
kind `quent_local_migration`, version 1, final sql array and differences array of
area/change/support/table [namespace,table] or null/name or null. It copies SQL from
the finalized record and identifiers from retained typed differences. It is not a
full source snapshot export, a raw expression serializer or an apply input.
The existing 16MiB checked encoder bounds output; write failures retain the fixed
DetailError contract, no fallback. Existing CRUD local source bytes stay unchanged.
Sensitive getters are an explicit trusted application boundary, not protection
against application code deliberately logging secrets.

Finite validation results and remaining work: [evidence](migration-plan-v1-evidence.md).

Current QT-AI-11 PR3: [explicit human-controlled finite application](migration-apply-v1.md), [finite evidence](migration-apply-v1-evidence.md). Historical PR1/PR2 absence statements retain their original scope; no full runtime_pending marker is cleared.
