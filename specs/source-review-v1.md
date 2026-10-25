Current PR3: [bounded CI/SARIF/diff/expiring baseline contract](review-ci-v1.md). Earlier PR-specific absence statements are historical.

# QT-AI-08 planned PR2: finite Rust source candidates

`quent/source-review` exposes pure `source_review::review(&[u8])`. CLI feature
`source-review` enables exactly `review rust --json|--pretty FILE`, independently
of the existing `review` feature. Both can be enabled together. Default facade/CLI
remain Core-only; Core has zero dependencies. This feature adds optional exact
`syn =2.0.119` (parsing/full/visit, defaults off), plus the already-pinned serde and
serde_json. The only lock change is quent's direct syn edge. Syn/proc-macro2/
unicode-ident versions already existed in the lock. No driver, runtime, clap,
compiler internals, nightly, filesystem or execution dependency is added to the
library. Rust 1.94/edition 2024/publish=false and existing pins are unchanged.

## Finite syntax and identity boundary

The parser is syn's stable-library syntax parser, **not rustc or a type checker**.
A successful parse says only that this bounded input was accepted by syn. It does
not establish Rust edition/feature legality, successful compilation, name/module
resolution, type checking, Quent identity, SQL, argument correspondence or runtime
behavior. No input is executed or built. There is no cargo/rustc/build.rs/proc-macro
invocation, macro expansion, cfg evaluation, module/repo exploration, environment,
clock, network, catalog, provider or DB capability in the library.

| Syntax | Observation and limitation |
| --- | --- |
| Direct `quent_core::Draft::select(t,p)`, `insert(t,d)`, `update(t,d)`, `delete(t)`; optional leading `::` | Candidate family only; exact path segments and arity, no generic path arguments |
| Contiguous direct receiver chain `.filter(p)`, `.order(o)`, `.limit(n)`, `.count()` | Syntax-only chain; exact arities and no method turbofish. Count candidate requires a select root. Method ordering/repetition/types/values are not API validated |
| No filter in recognized UPDATE/DELETE chain | `missing_where=candidate`, never actual Inspection Blocked or a proven actual operation |
| Filter present | `present_syntax`, `missing_where=not_established`; no safe WHERE/SQL/tenant/key/permission proof |
| Arguments, paths, literals, arrays, tuples, blocks, functions and locals | Traversed syntactically, no value evaluation or receiver inference; arguments always unverified |
| Nested recognized expressions (including arguments, closures, conditions, inline modules and function bodies) | Visited, including candidates mixed with unknown constructs; duplicate receiver-spine roots suppressed only within recognized chains |
| Macros including macro_rules/invocations; derive/attributes/cfg/cfg_attr | Unexpanded/unevaluated diagnostic. Opaque token contents are not executable expressions or hidden candidates; whole-input incompleteness retained |
| use aliases, local same-name module/type/function, re-exports, other crates, unresolved receivers/traits, helper/wrapper calls | Names/receiver unresolved. Full spelling still NEVER establishes identity even without visible shadowing |
| Dynamic arguments, branches/loops/match/closures/async/try | Arguments/control-flow unresolved; no dataflow or path feasibility inference |
| Any other parsed syntax | Unsupported/unresolved observation where visited; global identity/context remain unverified regardless of diagnostics |

Strings, byte/C/raw strings, chars, lifetimes and comments are interpreted by syn,
not a regex candidate search. A fake builder inside literal/comment is not a
candidate. Macro/attribute token contents remain opaque and explicitly unknown.
Unknown expressions in the same file cannot be erased by a recognized candidate.
Diagnostics describe finite categories, not a complete list of every Rust syntax
kind or an exact count of unknown sites. No unsupported feature promotes coverage.

## Parser envelope before ANY parser call

The CLI's same-opened-handle regular-file reader still reads at most 65,536 actual
bytes (+1 for overflow detection). Library first checks bytes, then its stricter
4,096-byte **parser envelope**, before UTF8 validation or syn. Larger valid files
are deliberately Budget, including whitespace-only files. This limited PR supports
small explicit source fragments, not typical whole repositories or arbitrary files.

The preflight is a flat byte scan. It does **not strip or lex strings/comments**:
all their bytes count. It accepts at most 512 non-ASCII-whitespace bytes, 48 ASCII
punctuation bytes (including underscores and quotes) and 16 total grouping bytes
`()[ ]{ }` (without the spaces). These are upper bounds, not measured lexical token
or delimiter-balance counts. Every ordinary lexical token consumes at least one
counted byte; any inserted doc-comment attribute has bounded source punctuation
and text. A conservative token ceiling is 4,096 (eight times significant bytes),
including synthetic doc-comment attribute tokens. The total raw grouping ceiling
bounds source delimiter nesting to eight pairs, including macro tokens; synthetic
doc-comment attribute groups add at most one level. The total punctuation ceiling also bounds generic,
unary/reference/type/operator recursion even without braces. Recursive keyword
sequences are additionally bounded by the 512 non-whitespace bytes. Strings,
chars, lifetimes, raw literal hashes and nested comments can only spend these
budgets sooner; quoted/comment braces are never subtracted and never confer a
false nesting guarantee. Legitimate snippets with many punctuation bytes refuse.
No standalone lexical parser, unbounded recursive scanner, catch_unwind stack
protection or parser-error formatting is used.

All source/literal/comment/raw-string bytes are <=4,096; significant content is
<=512 bytes. Token spellings, AST construction/destruction and parser recursion
are bounded before parsing even on invalid syntax. Runtime traversal additionally
refuses over 16 items, 512 expression nodes or 16 operation candidates, without
returning a truncated summary. `nodes` counts visited expressions (including path
expressions), `items` includes nested items. Other syntax nodes are bounded by the
pre-parser envelope rather than counted as expression nodes. At most ten distinct
diagnostic codes are possible; deduplication retains every category. These limits
apply together. No promise of an OS-wide deadline or arbitrary thread-stack size
is made. Product CLI subprocess watchdog tests kill **and wait** on timeout; they
exercise large/deep inputs refused before parsing and near-envelope inputs passed
to syn, rather than interpreting process abort as an acceptable error.

## Output-only wire, precision and exits

New independent output kind `quent_rust_review`, integer version 1, closed
[documentary schema](source-review-v1.schema.json). The three existing
`quent_review` v1 input/output wires, fixtures and exits are unchanged. This new
kind is not added as an accepted old-wire authority input. Review has private
fields and Serialize only, no Deserialize/restoration/conversion/execute API.
Debug/Display/JSON/pretty expose only fixed codes, states, precision and counts.
No source, SQL, identifiers, path, span, line, snippet, token, value, free reason,
parser/OS error, hash or credential is retained. Error is fixed Invalid/Budget,
with no source cause. CLI's existing redacted panic hook still applies with
backtrace/tracing environments. No source panic or tracing payload is evaluated.

| Entire supplied input | State / coverage / code | Exit |
| --- | --- | --- |
| Any recognized candidate, including mixed unknown sites | partial / partial / syntax_candidates | 3 |
| Parsed nonempty items/attributes, no recognized candidates | unsupported / unsupported / unsupported | 4 |
| Empty/whitespace/non-doc-comment input | unknown / unknown / no_candidates | 3 |
| Invalid UTF8 or parser refusal within envelope | fixed QUENT_REVIEW_INVALID stderr, empty stdout | 2 |
| Any resource budget exceeded | fixed QUENT_REVIEW_BUDGET stderr, empty stdout | 2 |
| Feature disabled (valid grammar) | fixed QUENT_REVIEW_UNSUPPORTED stderr, empty stdout, no file open | 4 |
| Input/output I/O failure | fixed QUENT_REVIEW_IO stderr; input failure has empty stdout | 8 |

Risk is always `unknown`; no source Complete/low/accepted/exit0/Blocked exists.
Syntax `parsed`, candidate `syntax_only`, identity/SQL/native/model/binding/policy/
tenant/freshness/runtime independently `unverified`. Trusted capture is
`not_implemented`, with no JSON/annotation claim入口. It is optional and not
implemented in this PR. There is no app context/current/mapping/ScopeFacts
constructor, plan lowering, SQL renderer, binder, Inspection or execution path.
PR1 finite plan-data Core analysis, explicit typed app review and the manifest's
four domains remain separate. MissingWhere source candidates never replace them.

Fixed diagnostic register: identity_unverified, context_unverified, no_candidates,
macro_unexpanded, attribute_unevaluated, names_unresolved, control_flow_unresolved,
receiver_unresolved, arguments_unverified, unsupported_syntax. Category order is
stable traversal/first occurrence, without arbitrary diagnostic text. There is no
SARIF/diff/baseline/suppression (PR3), sensitive exporter or trusted capture format.

CLI grammar is validated before filesystem access, including unknown format,
extra/missing args, disabled features and `-`. No stdin, directory walk, implicit
module discovery or input writes. Existing Unix NONBLOCK|NOFOLLOW|CLOEXEC and
Windows reparse/device rejection, same-opened-handle regular-file check and actual
byte bound are reused. Intermediate symlink resolution, concurrent in-place
writes, OS I/O deadlines, filesystem snapshots, Windows live DB and physical server
authentication remain outside guarantees. Output write failure may leave partial
public bytes; it cannot produce successful review acceptance.

## Acceptance responsibilities

QT-C20 primary; C01/C08/C12/C15/C19/C24 boundaries. Independent handwritten
candidate/filter full-output fixtures pin JSON/pretty bytes and exit3. Counterexamples
cover macro/derive/cfg/alias/helper/shadow/other-crate/same-name method/dynamic flow,
mixed and nested unknowns, empty/no candidate, fake strings/comments, literal/
comment/lifetime/byte/C-string lexical forms, malformed/UTF8/budget/deep/unary/type/generic/macro
inputs. Product children have five-second kill+reap watchdogs. Existing redacted
panic injection is test-harness-only. Tracing and TCP detectors each have real
positive controls. Source abort/panic/FS-call spellings are parsed as data; no
execution or build is performed. Structural absence of DB/provider/clock/env APIs
is separate from detector evidence and never a fabricated transaction trace.

Shared diagnostic-checked compiler harness retains 179 negatives/nine controls;
adds ten authority/restoration negatives and one compiling Serialize/direct-builder fixture control,
for 189/10. Fresh generated consumer retains 76 negatives/eight controls across six
invocations. Both OSes run source-only and existing review combinations in CI;
Unix FIFO/device/symlink/replacement and Windows device/reparse/replacement tests
are retained for old inputs and separately added for Rust input. Historical 130
feature checks are retained; 64 source overlays and two cross-feature checks yield
196. Twenty output combinations are retained, five source combinations added.
Hosted CI retains 50 jobs per push/PR event, adds ten facade and six CLI jobs,
yielding 66 per event (132 across both), evaluated only on the actual head.

New source write/Tx/live workflows are nonapplicable. Existing Strict/CRUD/
validated/atomic/catalog/CLI/FIFO/codegen/patch/regen still require new digest-pinned
PG16.14/MySQL8.4.8 disposable regressions with isolated namespaces/roles/settings
and verified container/anonymous-volume removal. All 24 full runtime_pending cases
stay unchanged. Actual commands/counts/results, both-OS/DB/CI and unverified limits
are recorded separately in [measurements](source-review-v1-evidence.md) and the
fixed RelayWeft execution report. No merge, Issue closure, release or publish.
