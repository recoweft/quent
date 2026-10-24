# 受入責務台帳

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
