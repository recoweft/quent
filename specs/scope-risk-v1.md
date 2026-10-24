# QT-AI-01 PR2: conditional scope and structural risk v1

`Plan::inspect(Option<&ScopeFacts>)` borrows the private finalized Record. It reads
its captured operation/tree/dialect, alongside the original SQL and ordered typed
args; it neither renders again nor parses SQL/public output. `Inspection` retains
that record borrow and exposes the same explicit redacted `SensitiveSource`.
No DB/provider/catalog/EXPLAIN/clock is involved. Repeated inspection is deterministic.
Neither Plan success, Scope::Zero/Single, nor Risk::Low is an allow/permit, expected
row count, business authorization, execution safety or commit guarantee. No dispatch,
Tx, tenant gate, provider, OperationSpec, public view serializer, suppression or
baseline is implemented. The public result has no setters/reconstruction input.
Modifying copied display classifications cannot affect the private inspection.

## Explicit assumptions and identity

`ScopeFacts::application_asserted` owns a finite immutable table/path/alias,
dialect, column/type/nullability registry, Primary/Unique constraints and context.
All referenced predicate, projection, write and order columns must match that registry
exactly. Registry duplicate columns, missing/duplicate key components, cross-table
components, empty keys and oversized snapshots are refused with fixed PlanError.
An incomplete registry or mismatched table/path/alias/dialect/type/nullability at
inspection yields Unknown/InconsistentFacts. No column name (`id` included), schema
name, public digest, fixture metadata, fresh/allow claim or LIMIT establishes a key.
Multiple aliases for the same table are separate identities, never merged.

The app must explicitly assert `OrdinaryBaseTableWithoutEffects`: ordinary base
table semantics with no triggers, views, RLS, generated-column effects or other
unsupported semantics affecting this proof. Unknown/unsupported context yields
Unknown even for empty IN. This constructor does not authenticate the app, consult
a catalog or verify these statements. `Assumptions::ApplicationAsserted` accompanies
all proofs relying on accepted facts; there is deliberately no LiveVerified variant.
No target/session binding or current/live freshness is promised in this DB-free API.
Missing facts, inconsistent facts, unsupported context or budget failure report
Assumptions::None and Unknown completeness/scope. Conditional proofs require the
assertions to remain true in the intended database; no execution integration exists.

Initial key proof supports nonnullable native signed SQL integers only: PG
I16/I32/I64 and MySQL I8/I16/I32/I64, as already supported by finalize. Values must
have exactly the same native type as their column; no coercion is introduced.
Primary and Unique constraints have the same maximum-one property only when all
components meet this domain. Nullable unique, text/collation and MySQL unsigned
keys cannot establish a bound. An independent supported primary/unique key may
still bound the whole predicate even if another constraint is unsupported.

## Finite predicate rules

Scope describes an upper bound on matching target rows, never existence, exact
cardinality, affected rows, planner estimates or all indirect database effects.

| Scope | Meaning |
| --- | --- |
| Zero | no TRUE candidate under the finite rules and accepted context |
| Single | at most one matching row, conditional on asserted complete key |
| FiniteBound(n) | at most n distinct complete key tuples, n >= 2 |
| Broad | no finite key bound established for the supported abstraction; not a claim of many actual rows |
| Unknown | missing/inconsistent/unsupported facts, predicate or exhausted analysis budget |
| NotApplicable | INSERT is not a matching-WHERE scope proof |

Analysis walks the original AND/OR/NOT tree, using a bounded internal set of possible
TRUE branches. It does not rewrite the record or SQL and does not implement general
SAT/theorem proving. It expands AND by conjunction, discarding conflicts between
same-column supported signed literal equalities. OR retains every possible branch.
Each candidate key must have every component bound to a literal in every remaining
branch. Distinct full tuples are deduplicated internally, without exporting values.
If several independent keys bound all branches, the smallest bound is used.
`OR(key=1,key=2)` is FiniteBound(2); repeated identical tuples can prove Single.
Composite partial equalities, tenant alone, range and multiple IN are never Single.
Even singleton IN is deliberately not key equality in this version.

Independent complete key equality in AND can limit branches containing unsupported
conditions. That scope stays conditional and Completeness::Partial; risk is High.
An unknown OR branch without the same bound leaves the whole proof Unknown.
Supported comparisons/ranges and nonempty signed nonnullable IN are conservatively
unbounded unless an independent equality bounds them. No range contradiction
reasoning, NULL-IN simplification, text equality inference or NOT inversion occurs.

Empty IN has no TRUE candidates (false, including NULL input). Any comparison with
a NULL literal has no TRUE candidates (SQL UNKNOWN); direct NOT of that comparison
also has no TRUE candidates. Other NOT, IS NULL, nullable comparisons and text/bool
comparisons are unsupported abstractions. NOT never complements a set of TRUE
candidates: FALSE and UNKNOWN cannot be interchanged under SQL three-valued logic.
An empty branch set can prove Zero even when an unsupported conjunct was discarded.
This only proves an empty target truth set, not an actual DB result or authorization.

Completeness is independent of the bound: Complete means all surviving candidates
are covered using supported finite rules (conservative overapproximation, not a
complete SQL solver); Partial retains an unsupported predicate abstraction, including
when a separate key establishes an upper bound; Unknown means facts/domain/budget
prevented a proof. Broad/Complete is possible: this is not a finite proof or permission.
INSERT completeness is Unknown and scope NotApplicable with consistent facts.

Budgets: at most 256 registry columns, 16 keys, 16 components/key; finalize already
bounds predicate depth/nodes/children/arguments. The branch abstraction caps each
intermediate set at 256 branches and total conjunction candidate pairs at 65,536.
Exceeding either discards the proof with AnalysisBudget/Unknown; no truncated proof
is accepted. A bounding sibling does not bypass an exhausted analysis budget.

Fixed Evidence register: MissingFacts, InconsistentFacts, UnsupportedContext,
UnsupportedKey, AnalysisBudget, EmptyTruthSet, CompleteKeyTuples,
UnboundedPredicate, UnsupportedPredicate, NoPredicate, InsertShape. These describe
structural evidence, not source strings, live measurements or external decisions.

## Deterministic risk register

Risk is the closed Low/Medium/High/Destructive/Blocked classification. Findings are
fixed enum codes with fixed order; every finding currently returns suppressible=false.
No free-text reason/source-error/suppression input is accepted. MissingWhere takes
precedence over all other findings and cannot be downgraded by facts or display edits.

| Rule | Risk / findings |
| --- | --- |
| UPDATE/DELETE without WHERE | Blocked / MissingWhere only, always non-suppressible |
| INSERT | High / InsertUnverified; no insert safety or schema-policy proof |
| Other operation with Unknown scope or non-Complete analysis | UnknownProof appended first; High unless missing-WHERE block |
| Filtered UPDATE/DELETE Zero/Single, Complete | Low; still no permission |
| Filtered UPDATE/DELETE finite tuple bound >=2 | High / BulkWrite (after UnknownProof if partial) |
| Filtered UPDATE Broad | High / WeakPredicate |
| Filtered DELETE Broad | Destructive / WeakPredicate |
| SELECT | ReadProjection structural marker (no PII inference from names) |
| SELECT/COUNT without limit | ReadWithoutLimit after projection; Medium when proof Complete |
| SELECT/COUNT with limit and Complete analysis | Low; LIMIT does not change scope or establish a key |

MissingWhere is a structural block designation, not an implemented dispatcher.
No findings grant execution. Fixed codes/evidence/classifications are safe summary
vocabulary; they contain no SQL, tuple values, identifiers, source errors or free text.
ScopeFacts/Key/Inspection have redacted Debug and no Display/serde. Detailed SQL,
args and metadata remain accessible only through explicit sensitive getters.

## Evidence and limits

Core tests cover both dialects: complete/composite/unique key positives, same and
different tuple OR, key-plus-nonkey OR, independent AND, contradictions, empty IN,
NULL/direct NOT, unsupported NOT, nullable unique, partial key, tenant, range,
multiple IN, identity/type/nullability/context mismatch, missing facts, unsigned
refusal, budget exhaustion, fixed diagnostics, display-copy edits, redaction and
record immutability. An independent three-valued evaluator checks 196 nested
predicate shapes × 2 dialects against enumerated TRUE rows; this is a finite
regression oracle, not a proof for every SQL program or live server.

The common fixture adds 3 scope-only subscenarios (69 total across 24 case IDs).
Production inspection consumes 12 scenarios × 2 dialects from QT-C02–05/C07, with
independent expected scope/risk/args and unchanged original SQL/tree/metadata.
The harness separately supplies an explicit ordinary-base-table assertion: fixture
schema/fresh/allow never supplies context authority. Without that assertion it
checks UnsupportedContext. Full fixture verdicts include future policy/gate duties;
for example partial-key Blocked, nullable-unique Unknown, and read risk expectations
are not silently copied into current structural results. `email = NULL` can have
Zero truth set here, while nullable unique cannot establish Single. All full runtime
acceptance markers remain pending. QT-C07 lossless-values is skipped in the scope
bridge because decimal/missing has no supported operation; existing separate value
refusal tests cover it. Empty recorder observations demonstrate the test harness did
not use its recorder, not an absent dispatcher/Tx's behavior.

No real DB/server version, encoding/decoding, affected rows, triggers/RLS semantics,
provider, authorization, transaction, GUI or exported artifact was tested. Optional
driver checks verify dependency compatibility only. Ubuntu/Windows workflow results
must be read from the actual PR checks separately from local WSL tests.

## Local verification for this change

Ubuntu 24.04.5 / WSL2, Rust/cargo 1.94.0: fmt check, workspace build/test,
all-target/all-feature clippy with warnings denied, all-feature doc with warnings
denied, and each optional postgres/mysql + runtime-tokio + tls-rustls compatibility
check passed using --locked --offline. 40 tests passed (30 existing + 10 added:
7 Core inspection, 1 fixture inspection, 2 compile-fail), zero failures/ignored.
The scope bridge's one unsupported value scenario is a documented manual exclusion,
not an ignored Rust test. Core has no dependencies; default normal workspace tree
still contains only local crates. Cargo.toml/lockfile/MSRV/edition/publish settings
are unchanged. No Windows executable was invoked locally; hosted CI evidence is
reported separately on the PR. No runtime acceptance markers were cleared.
