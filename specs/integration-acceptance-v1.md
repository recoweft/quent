# QT-AI-14 PR1: integrated finite acceptance register

This register supplements [the original acceptance responsibilities](acceptance.md)
and [unchanged v1 fixtures](../fixtures/contracts-v1.json). Original expectations,
loader vocabulary and **all 24 runtime_pending** entries are unchanged. A passed
subcase never completes a full fixture case. The source links below describe current
finite capabilities, including historical evidence; prior work receipts/counts are
not measurements of this execution.

Work `2ec4c522-263d-47ad-ad47-8769b4ab0b37`, revision 1; execution
`703baafb-9aa6-40fc-b586-512dcfa7475d`. Exact final source head, commands, DB settings,
subcase results and CI jobset are recorded separately in this execution's RelayWeft
report and PR. This document is a source-to-contract register, not a future-head
success assertion. Freshness and application assertions never confer authority.

## Executable workflows and responsibility boundaries

`execution/tests/acceptance.rs` runs one tenant/current/request/business-table flow
on each dedicated engine: immutable operation → candidate → repeated public
`candidate.review()` → explicit acquisition → owned exact1 completion → independent
full-tenant observer → expectation-mismatch rollback → borrowed read/caller rollback
→ durable command on the same table → replay/conflict → artificial ledger corruption
refusal → fixed audit/outbox bundle → replay/conflict/duplicate-companion rollback →
real plaintext COMMIT-response-loss/reconciliation with before-send and ACK controls. It separately checks review expiry, provider deny/error/cancel and cross
request/session acquisition. Connect/setup/observer traffic is excluded from the
business counter. The synthetic registered adapter is an application assertion and
a controlled test response, not authentication of a user/provider/physical server.

The minimal repair selected by specification answer
`3fb12ab2-3987-41ab-8dfa-7a90ee6b342f` adds `ExecutableAuthorizationCandidate::review`
under existing `review`. It internally borrows the retained preparation through the
common inspection and returns only the existing owned public Review. There is no
new sensitive getter, Inspection exposure, planner, preparation/render pass, source
wire, mutable reviewed flag, restored carrier or execution capability. Repeated
review is DB/provider0 and cannot extend TTL or bypass current/permit/native checks.
An internal test-only observation seam compares pre-review SQL allocation, every
ordered argument and metadata with provider and native dispatch. Matching public
JSON alone would not prove correspondence. A separately prepared pre-review in the
initial prototype did not meet this requirement; provider-inside review is supporting
evidence only, not its replacement.

`examples/tenant_review.rs` is a DB-free synthetic walkthrough: production generation,
exact regeneration comparison, nominal key/repository read, three-state patch and
manifest-domain review and a separate constraint-free nullable-integer tail ADD
plan with Expand/NoWork phase checks. The migration source is supplied independently,
never restored from the keyed model or a Fresh manifest. The DB-free example does
not perform preflight, human acquisition or apply; the dedicated live suites exercise
those existing explicit boundaries. It deliberately uses the checked fixture application's
assertion helpers; replace those with authenticated current application supply in
an actual service. Patch inspection does not become an executable update. The real
public execution sequence is exercised in the live acceptance tests above.

Generated-v2 live subflows additionally bind the explicit dialect-specific current
policy, compare production regeneration bytes, inspect a three-state patch and
execute a generated read through the same retained record/native binder. A separate
observer confirms that the inspected patch did not change persisted big/text data.
The additional `mcp_stdio::generated_workflow` group uses that same fixture model/
current mapping and production generator, direct Spec compilation and a real child
running the existing registered Server/transport. Its compile output, offline
manifest review, forbidden tool refusals and detecting loopback control establish
a finite artifact-to-MCP workflow; the ordinary CLI still has no registration.

Retained command bundle, live manifest, migration and MCP suites retain their
finite integration boundaries. They are distinct workflows, not one distributed
transaction or proof that every supported API participates in the new tenant flow.
Migration's constraint-free nullable ADD subset cannot migrate the keyed business
table as though arbitrary DDL were supported. It uses a separate typed plan, exact
preflight/phase/target evidence, new finite observation, separate human decision and
independently supplied current assertions. PostgreSQL COMMIT/ROLLBACK ACKs differ
from MySQL acknowledged prefix/implicit commit/unknown. External human identity,
routing/exclusion services remain application responsibilities. MCP uses the same
PR50 server/stdio transport and real child tests; it never receives a DB session,
application execution candidate or provider capability.

## All original cases: expectation versus finite evidence

All rows retain documented / fixture_validated / runtime_pending. “Regression” is
a source responsibility until a command on the exact reported source is recorded.
New integration evidence is limited to the subcases named here; unrepresented
original expectations remain unverified, and listed unsupported features remain
unsupported even if neighboring tests pass.

| Case | Original expectation (unchanged) | Current capability/evidence source | New/retained finite subcases | Limits / remaining responsibility |
| --- | --- | --- | --- | --- |
| QT-C01 | DB/provider呼出し0。反復結果は時刻等を除き決定的 | [plan-ir-v1](plan-ir-v1.md), [typed-operation-spec-v1](typed-operation-spec-v1.md), [review-v1](review-v1.md) | acceptance: pre-acquisition retained-record review; tenant_review example; Core/typed tests | No implicit DB/provider; full original SELECT/compile fixture remains pending |
| QT-C02 | non-suppressible blocked、dispatch 0 | [scope-risk-v1](scope-risk-v1.md), [policy-preparation-v1](policy-preparation-v1.md) | common gate/policy regression | No structural block waiver |
| QT-C03 | tenant-wideでありsingle-rowではない | [scope-risk-v1](scope-risk-v1.md), [authorization-gate-v1](authorization-gate-v1.md) | acceptance: tenant-wide UPDATE remains High; explicit acquisition | Broad is not exactly-one; expectation checked after full result |
| QT-C04 | broad/unknown、id名から単一行にしない | [scope-risk-v1](scope-risk-v1.md) | scope/policy regression | Finite signed-integer schema subset; general key theorem proving unsupported |
| QT-C05 | 全枝を証明できずbroad/unknown | [scope-risk-v1](scope-risk-v1.md) | scope/policy regression | Every OR branch required; general SQL unsupported |
| QT-C06 | tenant漏れ拒否、auto時はcaller全体の外側AND | [policy-preparation-v1](policy-preparation-v1.md) | policy/dispatch regression | Outer AND tenant guard; no trusted tenant from wire |
| QT-C07 | SQL意味を保持。空INは有限のfalseへ正規化し結果を明記、未対応型を安全にしない | [plan-ir-v1](plan-ir-v1.md), [sqlx-dispatch-v1](sqlx-dispatch-v1.md) | native/NULL/empty-IN regression; generated example | Exact decimal/UUID/date/time/bytes execution unsupported; no float coercion |
| QT-C08 | trusted context/permitへ昇格しない | [authorization-gate-v1](authorization-gate-v1.md), [schema-manifest-v1](schema-manifest-v1.md) | acceptance: ordinary High refuses; manifest/review output nonpromotion | Application assertion is not authenticated identity |
| QT-C09 | 全batch dispatch 0 | [policy-preparation-v1](policy-preparation-v1.md) | batch preparation refusal regression | General runtime batch unsupported; no partial dispatch |
| QT-C10 | compileまたはruntime拒否、dispatch 0 | [typed-patches-regeneration-v2](typed-patches-regeneration-v2.md) | tenant_review three-state patch; fresh generator consumer/compiler regressions | Patch is inspection-only; no patch execution |
| QT-C11 | 変更・共有による漏れなし、Txへ正しいsnapshot継承 | [settings-context-v1](settings-context-v1.md), [atomic-results-v1](atomic-results-v1.md) | acceptance: exact tenant context, borrowed caller completion | Physical routing/current snapshot provenance remains application responsibility |
| QT-C12 | mismatch、dispatch 0、新handle必要 | [owned-validation-v1](owned-validation-v1.md), [authorization-gate-v1](authorization-gate-v1.md) | acceptance changed subject; reviewed candidate mismatch regressions | No hash/public-view equality substitutes for full content |
| QT-C13 | compile-fail/API拒否、再利用不能 | [owned-validation-v1](owned-validation-v1.md), [authorization-gate-v1](authorization-gate-v1.md) | owned_api compiler negatives + new review control/negatives | Ownership is not DB exactly-once |
| QT-C14 | 拒否。最終dispatch直前の変更も検知 | [authorization-gate-v1](authorization-gate-v1.md), [authorized-cancel-v1-evidence](authorized-cancel-v1-evidence.md) | acceptance review TTL/deny/error/cancel; existing deterministic PR45 controls | Real elapsed expiry and artificial exclusive-deadline seam distinguished |
| QT-C15 | SQL/識別名/値/reason/source errorの漏れなし、実行復元不可 | [output-boundaries-v1](output-boundaries-v1.md), [mcp-v1](mcp-v1.md) | acceptance new Review canaries; existing public output/stdio controls | MCP top-level /id reflection exception only; Windows detail export unsupported |
| QT-C16 | owned Tx rollback。borrowed Txにatomic保証なし | [atomic-results-v1](atomic-results-v1.md) | acceptance owned commit/expectation rollback, borrowed read; version/RETURNING regressions | Owned complete consumption differs from caller-owned unconfirmed borrowed Tx |
| QT-C17 | replay/IdentityConflictを区別、業務write二重化なし | [command-ledger-v1](command-ledger-v1.md), [command-bundle-v1](command-bundle-v1.md) | acceptance same-table first commit/replay/conflict; concurrent bundle regressions | External delivery and arbitrary operation sets unsupported |
| QT-C18 | OutcomeUnknown→同ID照合。不正ledgerを再実行しない | [command-concurrency-v1-evidence](command-concurrency-v1-evidence.md), [command-bundle-v1-evidence](command-bundle-v1-evidence.md) | acceptance artificial stored version corruption; retained network-loss/outbox regressions | Artificial corruption is not network loss; Unknown is not rollback ACK |
| QT-C19 | 対象別freshnessを表示、offline一致をliveとしない | [manifest-verify-v1](manifest-verify-v1.md) | tenant_review offline domains; catalog/CLI regression | Offline/source equality is not live freshness; each domain separate |
| QT-C20 | partial/unsupported、CIで無条件成功にしない | [source-review-v1](source-review-v1.md), [review-ci-v1](review-ci-v1.md) | source/CI regression | Syntax candidates do not establish resolved type/control flow or unconditional CI pass |
| QT-C21 | 非ANALYZE、明示取得、Planは暗黙取得なし、budget拒否はdispatch 0 | [query-observation-v1](query-observation-v1.md), [performance-plan-v1](performance-plan-v1.md) | request observation/budget/performance regression | Authorized/command/generated wrapper/performance/catalog/migration unsupported inside observation scopes; non-ANALYZE estimates Partial |
| QT-C22 | 拒否/限界表示、MySQL完全rollbackを装わない | [migration-apply-v1](migration-apply-v1.md), [migration-preflight-v1](migration-preflight-v1.md) | migration plan/preflight/apply regression and independent catalog observers | Only nullable signed-integer tail ADD; independent human/current attestations; PG Tx vs MySQL implicit prefix/unknown |
| QT-C23 | tool不存在または拒否、DB/provider呼出し0 | [mcp-v1](mcp-v1.md) | same server/transport actual child stdio + ordinary CLI refusal regression | Offline read-only; no CRUD/permit/apply/live; synthetic registration not product authentication |
| QT-C24 | 有界に拒否、旧receiver/低risk既定へフォールバックしない | [operation-spec-v1](operation-spec-v1.md), [mcp-v1](mcp-v1.md) | bounded parsers/FIFO/protocol/version/input regression | No legacy receiver/low-risk fallback; all unsupported wire remains refused |

## Reproduction and measurement rules

Rust/MSRV is 1.94.0 throughout. `tenant_review` requires `codegen,review`; no driver
or runtime. The new public candidate review requires `review` plus an existing
selected driver and `runtime-tokio`, without forcing these on default/Core. Live
acceptance tests need the same features; the complete suite uses all-features.
`owned_api` retains 417 negatives +24 controls and adds eight review/authority/
ownership negatives +one control (425 +25 total); generated consumers
are freshly generated and compiled, not inferred from checked-in bytes alone.
No CI matrix is reduced: 304 grid +8 continuation feature checks, three CLI tests,
34 facade +10 CLI feature sets with test/Clippy on both hosted OSs remain required.

```sh
cargo run -p quent --example tenant_review --no-default-features --features codegen,review --locked
cargo test --workspace --all-features --locked --no-run
bash scripts/test-dispatch.sh cargo test --workspace --all-features --locked -- --test-threads=1 --nocapture
bash scripts/check-features.sh
```

Only the existing digest-pinned PG16.14/MySQL8.4.8 loopback fixture script is used;
never production/DATABASE_URL. Limited disposable role/namespace, MySQL strict
modes/FOUND_ROWS/InnoDB and per-run owned container/anonymous-volume cleanup and
absence checks apply to failed runs too. Compiler/build runs are sequential.
Measurements distinguish business submission, auxiliary ledger/catalog SQL,
provider calls/responses, Tx attempts/ACKs and independent observer queries.
The request observation budget counts supported non-Tx SQL attempts, has no refund,
and refuses the next dispatch at zero budget; Unknown and N+1 suspicion differ.
PerformancePlan is explicit non-ANALYZE Partial/Estimated, not measured improvement
or authorization. Windows hosted DB-free is not Windows live DB or ACL evidence.

Full-command totals, unique new subcases and feature-duplicate executions must be
reported separately. Development failures (constructor Result handling; missing
row_decoded event; wrong regeneration summary accessor; dialect-specific current
policy mismatch; Clippy guard scope; stack overflow from inlining review temporaries
into the existing large authorization regression future; helper-only attempt also
failed, so that suite was restored byte-for-byte and added reviewed mismatch
coverage moved into the new tenant flow) and initial filtered
2-test smoke runs remain historical evidence, not full acceptance. No full case
completion, public release or Go compatibility follows from this register.

[Goquent semantic case comparison](goquent-case-comparison-v1.md).

## Named new subcases (counts are not full fixture completion)

The two new live test functions each contain 36 conceptual A subcases below.
The four G subcases extend each existing generated live test. Seven M subcases
extend one additional stdio group for each dialect with **no live DB**. Thus 47
conceptual subcases, with dialect repetition, are distinct from test-harness totals
and feature-repeat invocations. The original large authorization regression suite remains unchanged; additional
reviewed current-content/identity/dispatch mismatches are measured inside the new
tenant test, preserving the original default-stack regression behavior.

| IDs | Finite observation / counter boundary |
| --- | --- |
| A01 | repeated candidate review, provider/business/auxiliary/Tx0, same pre-review/provider/dispatch SQL allocation/full args/metadata |
| A02 | ordinary High execution refused, provider/business/Tx0 |
| A03 | independently changed subject after acquisition, business/Tx0 NotStarted |
| A04 | authorized owned exact1: provider1, business1, begin+commit ACK; full consumed write result, independent two-tenant observer |
| A05 | authorized expectation mismatch: provider1/business1, begin+rollback ACK; preceding value persists |
| A06 | borrowed read1, Uncommitted, library completion0, caller rollback1 |
| A07/A08/A09 | plain command first business1; same-ID replay0; altered-input conflict0; independent QCC1/QCR1/version1 |
| A10 | artificial stored-version corruption, lookup refuses LedgerInvalid/business0; no automatic new ID/re-execute |
| A11/A12/A13 | bundle first business/audit/outbox1/1/1; replay0/0/0; changed companion conflict0/0/0 |
| A14 | duplicate companion under new explicit durable ID: business1/rejected audit1/outbox0, rollback ACK; observer prior business/audit1/outbox1/ledger2 |
| A15/A16/A17 | real protocol before-COMMIT/lost-reply/pass controls: business1 each; server COMMIT ACK false/true/true, client completion unknown/unknown/acknowledged; fresh lookup business0 and saved false/true/true; observer matches |
| A18 | real elapsed TTL after review, repeated expired review cannot extend deadline; acquisition provider0/business0 |
| A19a/A19b/A19c | registered test response Deny/Error/Cancelled: provider1 each, business/Tx0 |
| A20a/A20b | reviewed candidate wrong request/session acquisition, provider/business/Tx0 |
| A21a–k/A22/A23 | independently changed subject/target/route/settings identity+generation/policy identity+generation/schema identity+generation/tenant/policy contents (same generation included), wrong dispatch request and substituted operation: acquisition provider1 each, business/Tx0 |
| G01/G02/G03/G04 | production regenerate exact NoChange / inspection-only three-state patch SQL0 / generated read1 same record / independent prior big/text persisted |
| M01/M02 | direct and actual child compiled public view correspond / source manifest remains offline unknown, neither authority nor live |
| M03a–d/M04 | execute/apply/permit/live tool refusal / detecting network positive control, child connection0 and bounded parent kill/wait ownership |

These counts separate artificial ledger/observation seams and loopback protocol
fault fixtures from actual database commits. Setup/catalog observers and connection
initialization are not business submissions. The retained request observation and
migration suites report their own exact counters; their auxiliary/Tx measurements
must not be summed into these application business counts.

New tenant-flow aggregate counters are asserted separately from physical setup/
observer/connection traffic: main session business8 and ledger SELECT8/reserve3/
result-save2, all ledger ACKs; owned BEGIN10/ACK10, COMMIT5/ACK5, ROLLBACK5/ACK5.
Protocol sessions contribute business3 and ledger3×3, begin3 and commit attempts3
but client commit ACK1; independent fresh reconciliation contributes lookup3,
business0 and begin3/commit ACK3. Borrowed read1/caller rollback1 remains separate.
Provider calls19 include three response-rejection controls and thirteen acquired
permits subsequently rejected at the independently current dispatch boundary.
These are finite per-engine measurements, not all driver communications or a
production authentication claim. Final ledger observer sees four durable records;
failed duplicate and before-COMMIT controls leave no durable new identity.
