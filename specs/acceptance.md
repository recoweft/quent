Current QT-AI-05 PR2: [explicit finite catalog/CLI verification](manifest-verify-v1.md).
Earlier PR-specific absence statements below describe their historical baseline.

# 受入責務台帳

Current QT-AI-02 PR2 policy preparation, finite support/refusal matrix and evidence:
[policy preparation v1](policy-preparation-v1.md). Implementation-status paragraphs describe their named historical PR; the
integrated requirements remain current. Raw Plan/snapshot APIs are non-enforcing
and non-executable.

QT-AI-00 PR2: 全24件を [version 1 fixture](../fixtures/contracts-v1.json) に収録。
下表の入力・結果は契約の期待であり測定結果ではない。Goquent Cxxとの意味の対応は未検証。
QT-C21〜24も同じ形式でfixture_validated。実装責務は各行のowning issueに残る。

証拠 vocabulary: `quent-contract-evidence` version `1`（fixture本体のkind/versionとは別）。
`documented`: 契約シナリオあり。`fixture_validated`: loaderの形式・型・coverage検証対象。
`recorder_tested`: 記録器そのものの人工script試験（個別runtime caseの成立ではない）。
`runtime_pending`: production機能、compile-fail/API、実DB受入は未実装・未検証。
各caseは documented / fixture_validated / runtime_pending。recorder_testedは共通支援層のみ。
fixtureにruntime_passed等を追加してもloaderは拒否する。将来の測定証拠は別artifactに
head SHA・command・dialect/server/settings・結果・skip理由を結び、期待欄を書き換えて代用しない。

共通実測: loader/記録器/方言別消費雛形のRust integration tests。
[利用方法・budgets・実DB test責務](../crates/quent/tests/support/README.md)。
CLI/MCP/tracingへの詳細公開、production IR/planner/gate/SQLx dispatchは追加していない。

| Case | 入力/シナリオ | 期待 | Owning issue | case証拠 |
| --- | --- | --- | --- | --- |
| QT-C01 | SELECTのPlan/compile/review | DB/provider呼出し0。反復結果は時刻等を除き決定的 | QT-AI-01, QT-AI-06, QT-AI-08 | documented / fixture_validated / runtime_pending |
| QT-C02 | WHEREなしUPDATE/DELETE | non-suppressible blocked、dispatch 0 | QT-AI-01, QT-AI-02 | documented / fixture_validated / runtime_pending |
| QT-C03 | tenant一致のみのUPDATE | tenant-wideでありsingle-rowではない | QT-AI-01, QT-AI-02 | documented / fixture_validated / runtime_pending |
| QT-C04 | id>0、id IN複数、複合key一部 | broad/unknown、id名から単一行にしない | QT-AI-01, QT-AI-02 | documented / fixture_validated / runtime_pending |
| QT-C05 | key=1 OR status=active | 全枝を証明できずbroad/unknown | QT-AI-01, QT-AI-02 | documented / fixture_validated / runtime_pending |
| QT-C06 | (tenant=T1 AND key=1) OR key=2 | tenant漏れ拒否、auto時はcaller全体の外側AND | QT-AI-02 | documented / fixture_validated / runtime_pending |
| QT-C07 | NULL、NOT、空IN、nullable unique | SQL意味を保持。空INは有限のfalseへ正規化し結果を明記、未対応型を安全にしない | QT-AI-01 | documented / fixture_validated / runtime_pending |
| QT-C08 | JSON tenant、古いfresh claim、reasonのみ | trusted context/permitへ昇格しない | QT-AI-02, QT-AI-05, QT-AI-09 | documented / fixture_validated / runtime_pending |
| QT-C09 | batch内1行だけtenant不一致 | 全batch dispatch 0 | QT-AI-02, QT-AI-03 | documented / fixture_validated / runtime_pending |
| QT-C10 | tenant列更新、readonly/generated patch | compileまたはruntime拒否、dispatch 0 | QT-AI-02, QT-AI-07 | documented / fixture_validated / runtime_pending |
| QT-C11 | DB A/Bの異なるpolicyとtenant | 変更・共有による漏れなし、Txへ正しいsnapshot継承 | QT-AI-02, QT-AI-03 | documented / fixture_validated / runtime_pending |
| QT-C12 | Validate後の値/target/世代/subject変更 | mismatch、dispatch 0、新handle必要 | QT-AI-03, QT-AI-05 | documented / fixture_validated / runtime_pending |
| QT-C13 | handle再使用/Clone/serde/future破棄 | compile-fail/API拒否、再利用不能 | QT-AI-03 | documented / fixture_validated / runtime_pending |
| QT-C14 | 期限境界/cancel/認可error/別request decision | 拒否。最終dispatch直前の変更も検知 | QT-AI-03, QT-AI-09 | documented / fixture_validated / runtime_pending |
| QT-C15 | 公開JSON/Debug/error/SARIF/MCPへcanary値 | SQL/識別名/値/reason/source errorの漏れなし、実行復元不可 | QT-AI-04, QT-AI-08, QT-AI-13 | documented / fixture_validated / runtime_pending |
| QT-C16 | 期待行数不一致/楽観version/復号失敗 | owned Tx rollback。borrowed Txにatomic保証なし | QT-AI-03 | documented / fixture_validated / runtime_pending |
| QT-C17 | command同ID同入力/異入力/同時実行 | replay/IdentityConflictを区別、業務write二重化なし | QT-AI-10 | documented / fixture_validated / runtime_pending |
| QT-C18 | commit応答喪失/ledger破損/outbox再送 | OutcomeUnknown→同ID照合。不正ledgerを再実行しない | QT-AI-10 | documented / fixture_validated / runtime_pending |
| QT-C19 | schema/policy/code/live差分/未観測 | 対象別freshnessを表示、offline一致をliveとしない | QT-AI-05 | documented / fixture_validated / runtime_pending |
| QT-C20 | source macro/helper/cfg/動的query | partial/unsupported、CIで無条件成功にしない | QT-AI-08 | documented / fixture_validated / runtime_pending |
| QT-C21 | EXPLAIN/観測/budget | 非ANALYZE、明示取得、Planは暗黙取得なし、budget拒否はdispatch 0 | QT-AI-12 | documented / fixture_validated / runtime_pending |
| QT-C22 | migration未対応/古いpreflight/DDL失敗 | 拒否/限界表示、MySQL完全rollbackを装わない | QT-AI-11 | documented / fixture_validated / runtime_pending |
| QT-C23 | MCP write/raw/apply/permit要求 | tool不存在または拒否、DB/provider呼出し0 | QT-AI-13 | documented / fixture_validated / runtime_pending |
| QT-C24 | 不正JSON/未知version/巨大入力 | 有界に拒否、旧receiver/低risk既定へフォールバックしない | QT-AI-05, QT-AI-06, QT-AI-13 | documented / fixture_validated / runtime_pending |

## QT-AI-01 PR1 plan-only evidence

[Plan IR v1](plan-ir-v1.md) adds production planner tests for five families in both
dialects, with source-linked QT-C01/QT-C07 SQL/ordered args/metadata comparisons.
The recorder observes no calls; Core has no provider/DB dependencies or endpoint.
This is plan-only evidence, not dispatcher/Tx/risk/tenant acceptance. QT-C02–05
proof remains PR2 work, and every full runtime_pending marker stays unchanged.

## QT-AI-01 PR2 conditional structural evidence

[Scope/risk v1](scope-risk-v1.md) defines production inspection over the private
finalized record. QT-C02–05 and QT-C07: 12 common-source scenarios × 2 dialects
exercise fixed structural results, with separate application assertions, independent
expectations and unchanged SQL/args/tree/metadata. Three positive/finite OR scenarios
were added (69 scenarios total). Seven Core inspection tests include finite three-valued
regression checks, supported positives, unknown/unsafe counterexamples, immutable
results and canary/compile-fail boundaries. Full policy/dispatch/Tx/live-schema/DB
acceptance is absent; none of the 24 runtime_pending markers is cleared. The bridge
skips QT-C07 lossless-values (unsupported decimal/missing, tested as refusal elsewhere).
Fixture full expectations are not current production verdicts: the guide records the
limited structural distinctions, including Zero truth set versus nullable-key proof.

## QT-AI-02 PR1 snapshot foundation

See [immutable settings/application snapshots](settings-context-v1.md) for implemented
carrier APIs, support/refusal table, exact owned content, Strict default and the
application authentication responsibility. QT-C11 has DB-free A/B isolation and
preparation inheritance evidence; QT-C08 claim nonpromotion, QT-C12 exact content/
target/subject/generation comparison and QT-C15 redaction/compile-fail evidence are
limited to this carrier boundary. Raw snapshot declarations alone are not enforced; the current preparation API
enforces the finite subset described in the policy preparation guide. ScopeFacts
remains ApplicationAsserted; Plan/inspect retain the same private finalized record.
All full runtime_pending markers stay pending. Real executor/Tx binding and immediate
pre-dispatch revalidation belong to QT-AI-03+, tenant/column enforcement to QT-AI-02
PR2. No runtime dispatch/refusal/rollback or live database results are claimed.

## QT-AI-03 PR1 runtime boundary

[Immediate SQLx dispatcher](sqlx-dispatch-v1.md) adds the current shared Strict gate,
owned-session context binding, finite native adapters and bounded redacted results.
Its support/refusal and evidence tables supersede historical statements above about
the absence of execution. Raw Plan/source/preparation remain non-executable inputs.
INSERT remains InsertUnverified/High and is refused without a provider. All full
runtime_pending markers remain: limited live PR1 evidence is recorded separately
from PR2 handles, PR3 Tx/atomicity and QT-AI-09 authorization responsibilities.

## QT-AI-03 PR2 boundary

[Consumed validation](owned-validation-v1.md) records the current exact-operation/
context/session/deadline checks, ownership and async stage contract. This adds finite
QT-C12/C13/C14 evidence through the same production dispatcher without changing
Core planning, PR1 refusal/native gates or any full runtime_pending marker.

## QT-AI-03 PR3 finite responsibilities

[Atomic results v1](atomic-results-v1.md) records the production API, state/guarantee
tables, dialect semantics, actual dedicated-DB observer evidence, test-only injection
and explicitly unverified network completion faults. C16 has finite owned success/
rollback/version/RETURNING and borrowed non-atomic evidence. C11 snapshot inheritance
and C12–14 exact identity/input/expiry/consumption/cancellation are checked through
the same dispatcher; C02/C06/C07/C10/C15 retain existing regressions and new version/
projection checks. Original 87 tests remain, with four Core IR tests added; the API
harness has two compiling controls and 29 checked negative programs. This does not
clear any of the 24 full runtime_pending markers or establish provider/command
responsibilities. Local and hosted measurements are reported separately.

## QT-AI-04 PR1 finite output evidence

[Public view schema, registers, tests and remaining responsibilities](public-views-v1.md)
records QT-C15 production output canaries/schema/restoration rejection, with
C01/C08/C12/C13/C14/C16 supporting structural and live gate regression evidence.
The existing fixture is unchanged: its exact/low expectations are documentary,
not promoted to live/authorization claims. All 24 runtime_pending markers remain.
CLI/tracing/SARIF/MCP and detailed export/Windows ACL are not covered by PR1.

## QT-AI-04 PR2 finite output integration

[Output boundaries v1](output-boundaries-v1.md) inventories every current public
surface and distinguishes application-local SensitiveSource export from public JSON,
sealed tracing and the non-success CLI. QT-C15 adds actual source/newfile/typed-data,
owner/mode, no-overwrite/race and refusal evidence. Real PostgreSQL NOTICE/exception
canaries and raw-driver controls test private driver poll/drop suppression; this does
not expand trigger-backed DB semantics support. C01/C02/C08/C12–16 retain complete
private gate/handle/atomic regression; display/export never supplies permission.
Windows export is unsupported/fail-closed, not an ACL guarantee. Native CI protection
primitive tests are isolated from the product CI refusal. MCP tools, SARIF/review/
manifest parsers and remote application authentication remain owning-issue duties.
All 24 full runtime_pending markers and original documentary fixture expectations
remain unchanged. Local and hosted measurements are recorded separately.

## QT-AI-05 planned PR1 manifest boundary

[Manifest source v1](schema-manifest-v1.md) adds optional DB-free finite source/wire,
independent domain canonical/fingerprint and offline evidence. C19/C24 have bounded
production API positive/negative evidence, with C01/C07/C08/C12/C15 boundary coverage.
Source parsing/equality never supplies live catalog/authorization or trusted facts;
existing Strict/context/validated/atomic gates remain unchanged. Full runtime_pending
stays on all 24 fixture cases. PR2 owns explicit live catalog, drift and CLI verify;
codegen and business authority remain with their owning Issues. Measured commands,
exact head and CI/DB regressions are recorded in the execution/PR report.

## QT-AI-05 PR2 finite catalog boundary

C19/C24 add explicit real catalog, bounded comparison and strict CLI workflow;
C08/C12/C15 add private observation provenance, exclusive monotonic expiry, exact
current content/session and source-free nonpromotion boundaries. See the linked
reader matrix for supported defaults/privileges and PG/MySQL snapshot differences.
Existing CRUD/validated/atomic gates are unchanged. All 24 full runtime_pending
markers remain. Only actually executed commands and measured subcases in the PR
and execution report are evidence; the fixture expectations are unchanged.

Revision 2 C24/C15 input-boundary evidence adds real product subprocess tests for
writerless Unix FIFO in all three verify inputs, special-file/final-symlink refusal,
opened-file replacement races, exact byte limits and a detecting TCP positive
control for config rejection before connect. Five-second watchdogs kill/wait and
fail on hangs. Unix NONBLOCK is a FIFO-open guarantee, not an OS/filesystem deadline.
Windows file regressions run in hosted CI; Unix FIFO and local WSL evidence do not
establish Windows live support. No full runtime_pending marker is cleared.

## QT-AI-06 PR1 bounded source boundary

[OperationSpec v1](operation-spec-v1.md) defines C01/C24 production syntax/range
validation and C07/C08/C11/C12/C15 finite NULL/native/role/snapshot/output
subcases. Source and role-checked source cannot become operations, context or
validated/atomic/observation handles through public conversion or JSON. No typed
compiler or execution connection is implemented. All 24 full runtime_pending
markers remain unchanged; command/exact-head CI measurements belong to the report.

## QT-AI-06 planned PR2 typed boundary

[Typed compilation contract](typed-operation-spec-v1.md) adds finite C01/C07/C08/
C11/C12/C15/C24 evidence: exact full application mapping, native width/sign/NULL/
operator matrix, fixed source-free diagnostics/actions, one common finalization,
actual owned read and same-record dispatcher observations. C02–06/C10/C13/C14/C16
retain common policy/validated/atomic regressions. Typed compilation never replaces
business permission or weakens High/Unknown refusal; soft-delete remains subject
to existing partial-analysis refusal. All 24 full runtime_pending markers and
documentary fixture expectations remain unchanged. Commands, actual counts, new-head
CI, failures/skips and disposable DB cleanup are recorded in the revision report.

## QT-AI-07 planned PR1 finite generated consumer

[Generated model contract and evidence matrix](typed-codegen-v1.md) adds limited
C01/C07/C08/C10(read-only surface)/C11/C12/C15/C19/C24 evidence. Production inputs
and independently handwritten API/type/SQL/native/result oracles accompany byte
correspondence fixtures. Fresh production-generated consumers test model mixing,
native/NULL/partial-key/tenant/private/serde/authority refusals with diagnostic
codes and compiling controls. Live generated reads compare inspected/dispatched
SQL allocation and ordered args, native+NULL and partial projection rows, context/
mapping/High/policy refusal and result-budget quarantine without retry. Existing
Strict/CRUD/owned/atomic/catalog/CLI regressions remain required. Full C10 patch
behavior and regen management belong to PR2. Every original full runtime_pending
marker stays unchanged. Measurements/CI belong to this revision's execution report.

## QT-AI-07 PR2 finite patch/regeneration boundary

[Explicit generator v2](typed-patches-regeneration-v2.md) adds model-specific owned
three-state patches, native/NULL/budget/duplicate/empty checks and protected-setter
absence, compiled through existing common preparation and the retained record for
inspection only. Pure exact previous/proposed/existing byte comparison never opens,
writes or deletes a file and cannot grant ownership or apply permission.
C10 gains typed and common-policy protection evidence; C01/C07/C08/C11/C12/C15/C19/
C24 gain the finite SQL/args/metadata, context/mapping, source-free output, version/
domain/CRLF/conflict and budget subcases in patch_regeneration and fresh consumer.
C02–06/C13/C14/C16 retain the existing dispatcher/atomic boundary: CompiledPatch
has no execution conversion, so there is no new send/session/expiry/Tx path.
All 24 full runtime_pending entries remain unchanged. New head results are
execution evidence, not inferred from this ledger.
