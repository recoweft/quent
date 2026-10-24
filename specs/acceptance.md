# 受入責務台帳

PR1では以下の期待を定着する。全件runtime未実装・未検証。fixture/記録器はQT-AI-00 PR2、実際のdispatch/Tx試験はowning issueが担当する。両方言（PostgreSQL/MySQL）を対象に、入力・schema/context・正例/反例・診断・dispatch数・Tx結果・precision・未検証範囲を実装PRで記録する。下表の結果は期待であり測定結果ではない。Goquent Cxxとの対応は未検証。

| Case | 入力/シナリオ | 期待 | Owning issue | 実測 |
| --- | --- | --- | --- | --- |
| QT-C01 | SELECTのPlan/compile/review | DB/provider呼出し0。反復結果は時刻等を除き決定的 | QT-AI-01, QT-AI-06, QT-AI-08 | 未検証 |
| QT-C02 | WHEREなしUPDATE/DELETE | non-suppressible blocked、dispatch 0 | QT-AI-01, QT-AI-02 | 未検証 |
| QT-C03 | tenant一致のみのUPDATE | tenant-wideでありsingle-rowではない | QT-AI-01, QT-AI-02 | 未検証 |
| QT-C04 | id>0、id IN複数、複合key一部 | broad/unknown、id名から単一行にしない | QT-AI-01, QT-AI-02 | 未検証 |
| QT-C05 | key=1 OR status=active | 全枝を証明できずbroad/unknown | QT-AI-01, QT-AI-02 | 未検証 |
| QT-C06 | (tenant=T1 AND key=1) OR key=2 | tenant漏れ拒否、auto時はcaller全体の外側AND | QT-AI-02 | 未検証 |
| QT-C07 | NULL、NOT、空IN、nullable unique | SQL意味を保持。空INは有限のfalseへ正規化し結果を明記、未対応型を安全にしない | QT-AI-01 | 未検証 |
| QT-C08 | JSON tenant、古いfresh claim、reasonのみ | trusted context/permitへ昇格しない | QT-AI-02, QT-AI-05, QT-AI-09 | 未検証 |
| QT-C09 | batch内1行だけtenant不一致 | 全batch dispatch 0 | QT-AI-02, QT-AI-03 | 未検証 |
| QT-C10 | tenant列更新、readonly/generated patch | compileまたはruntime拒否、dispatch 0 | QT-AI-02, QT-AI-07 | 未検証 |
| QT-C11 | DB A/Bの異なるpolicyとtenant | 変更・共有による漏れなし、Txへ正しいsnapshot継承 | QT-AI-02, QT-AI-03 | 未検証 |
| QT-C12 | Validate後の値/target/世代/subject変更 | mismatch、dispatch 0、新handle必要 | QT-AI-03, QT-AI-05 | 未検証 |
| QT-C13 | handle再使用/Clone/serde/future破棄 | compile-fail/API拒否、再利用不能 | QT-AI-03 | 未検証 |
| QT-C14 | 期限境界/cancel/認可error/別request decision | 拒否。最終dispatch直前の変更も検知 | QT-AI-03, QT-AI-09 | 未検証 |
| QT-C15 | 公開JSON/Debug/error/SARIF/MCPへcanary値 | SQL/識別名/値/reason/source errorの漏れなし、実行復元不可 | QT-AI-04, QT-AI-08, QT-AI-13 | 未検証 |
| QT-C16 | 期待行数不一致/楽観version/復号失敗 | owned Tx rollback。borrowed Txにatomic保証なし | QT-AI-03 | 未検証 |
| QT-C17 | command同ID同入力/異入力/同時実行 | replay/IdentityConflictを区別、業務write二重化なし | QT-AI-10 | 未検証 |
| QT-C18 | commit応答喪失/ledger破損/outbox再送 | OutcomeUnknown→同ID照合。不正ledgerを再実行しない | QT-AI-10 | 未検証 |
| QT-C19 | schema/policy/code/live差分/未観測 | 対象別freshnessを表示、offline一致をliveとしない | QT-AI-05 | 未検証 |
| QT-C20 | source macro/helper/cfg/動的query | partial/unsupported、CIで無条件成功にしない | QT-AI-08 | 未検証 |
| QT-C21 | EXPLAIN/観測/budget | 非ANALYZE、明示取得、Planは暗黙取得なし、budget拒否はdispatch 0 | QT-AI-12 | 未検証 |
| QT-C22 | migration未対応/古いpreflight/DDL失敗 | 拒否/限界表示、MySQL完全rollbackを装わない | QT-AI-11 | 未検証 |
| QT-C23 | MCP write/raw/apply/permit要求 | tool不存在または拒否、DB/provider呼出し0 | QT-AI-13 | 未検証 |
| QT-C24 | 不正JSON/未知version/巨大入力 | 有界に拒否、旧receiver/低risk既定へフォールバックしない | QT-AI-05, QT-AI-06, QT-AI-13 | 未検証 |
