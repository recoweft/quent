# Quent — AI時代に対応したRust ORM 統合仕様・Issue/PR計画 v1

Current adoption entry: [finite v0.x support/refusal matrix](adoption-v1.md),
[reproduction](reproduction-v1.md), [publication holds](release-checklist-v1.md).
The specification baseline below is not a claim that every planned capability is implemented.

Current QT-AI-02 PR2 policy preparation, finite support/refusal matrix and evidence:
[policy preparation v1](policy-preparation-v1.md). Implementation-status paragraphs describe their named historical PR; the
integrated requirements remain current. Raw Plan/snapshot APIs are non-enforcing
and non-executable.

作成日: 2026-10-23（日本時間）
状態: 実装前の仕様ベースライン。連携反映日: 2026-10-24（日本時間）。
対象: recoweft/quent / base main
RelayWeft: quent（project ID 8f174e78-f969-4bf8-b173-f73c57c17bd4）
管理ID: QT-AI-00〜QT-AI-14 / 15 Issue・33 planned PR
本書は製品仕様ベースライン。運用履歴: 先行work revision 2はPR1のみを許可した。PR1 (#16) 統合後、work 7b3e9b95-154c-4f1c-bac6-4bf4a980c741 revision 1はPR2実装・push・PR作成を許可する。旧段階の待機文は後続の明示依頼を制限しない。merge/releaseは今回も対象外。

## 1. コンセプトと仕様の根拠

Quentは、AIが生成したDB操作を、人間とツールが実行前に理解・検証・レビューでき、アプリケーションが実行を制御できるRust ORMとする。通常のアプリケーション開発にも使えるCRUD、型付きmodel、projection、更新patchを提供する。LLMはCoreの必須依存にしない。

基本方針はPlan-first、SQL-visible、Policy-aware、Deterministic、Staleness-aware。RiskLowは構造上の低リスクであり、業務的な正しさ、認証、利用者の権限、commit成功を保証しない。

別チャットでのGoquent仕様を設計の出発点とし、現在のGoquentの契約と改善履歴を反映した。Rust版固有の既存仕様は今回の取得範囲では確認できず、本書のRust構成と分割は今回新たに整理した設計判断である。GoのAPI、旧互換動作、wire形式、HMAC方式をそのまま移植することは要件にしない。

確認した参照:
- Goquent main: b60b6b73b0d0c997dc2bbd74971de4112a398606
- Quent main: bc7fe02066ccd854965140750cfec38f86f35c14
- Quentは確認時点でREADME/LICENSE/gitignoreのみ。Rust workspace・既存仕事なし。
- Goquentの外部認可は上記mainでは契約PR1まで。実行統合済みとは扱わない。
- Goquentロードマップの歴史的チェックリストは現行の完了台帳ではない。

固定参照:
- [Goquent contracts v3](https://github.com/recoweft/goquent/blob/b60b6b73b0d0c997dc2bbd74971de4112a398606/specs/goquent_ai_contracts_v3.md)
- [Vision](https://github.com/recoweft/goquent/blob/b60b6b73b0d0c997dc2bbd74971de4112a398606/docs/vision.md)
- [Plan binding](https://github.com/recoweft/goquent/blob/b60b6b73b0d0c997dc2bbd74971de4112a398606/docs/validated-plan-binding.md)
- [Tenant policy](https://github.com/recoweft/goquent/blob/b60b6b73b0d0c997dc2bbd74971de4112a398606/docs/tenant-policy.md)
- [Public views](https://github.com/recoweft/goquent/blob/b60b6b73b0d0c997dc2bbd74971de4112a398606/docs/public-plan-views.md)
- [Typed repositories](https://github.com/recoweft/goquent/blob/b60b6b73b0d0c997dc2bbd74971de4112a398606/docs/typed-repositories.md)
- [Command identity](https://github.com/recoweft/goquent/blob/b60b6b73b0d0c997dc2bbd74971de4112a398606/docs/command-identity.md)
- [External authorization contract](https://github.com/recoweft/goquent/blob/b60b6b73b0d0c997dc2bbd74971de4112a398606/docs/external-authorization.md)
- [SQLx Transaction公式API](https://docs.rs/sqlx/0.9.0/sqlx/struct.Transaction.html)
- [Rustdoc公式・documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)

## 2. 採用するRust構成

| 項目 | v1での設計 |
| --- | --- |
| 言語 | stable Rust / edition 2024。nightly必須機能を使わない |
| toolchain / MSRV | QT-AI-00 PR1で環境・依存の実際の最小版を検証しrust-versionとtoolchainを固定。未知の最低版を宣言しない |
| 実行 | async-first、Tokio + SQLx。SQLx 0.9系を第一候補に互換性・MSRV・TLSをPR1で確認して固定 |
| DB | PostgreSQL / MySQL。最初から両方のテスト基盤を設ける。SQLiteはv1の対象外 |
| Core | quent-core: IR、値、schema、policy、plan、診断、public view。DB/runtimeなしでbuild可能 |
| Library | quent: facade、型付きAPI、DB/Tx、private dispatcher、SQLx adapter。postgres/mysqlをfeature分離 |
| CLI | quent-cli: review/manifest/codegen/migration/MCP。ライブラリCoreと同じvalidator/plannerを使う |
| codegen | schema/manifestから明示生成。初期版では汎用derive proc-macroを必須にしない |
| 既定feature | core利用にDB driverを要求しない。接続機能は選択したadapter/runtime/TLSを明示 |
| 状態 | private fieldと不変snapshot。共有は不変Arc等に限定。mutableなglobal registryを置かない |
| 検証済み操作 | 非Clone・非Copy・非Serialize/Deserialize、所有権を消費するhandle |
| 品質 | fmt、clippy、unit/doc/compile-fail、feature matrix、両DB integration |
| 開発環境 | RelayWeftのUbuntu/WSLプロジェクトを主対象とし、WindowsのCore/CLI buildも検証 |

crate名や製品名の登録・商標・crates.io空き状況は未確認。今回publishはしない。依存のpatch版や実DBイメージは実装開始時の検証値をpinし、本書では未検証の最新版を断定しない。

## 3. 実行モデルと信頼境界

通常の流れは、typed builder / generated repository / OperationSpec → immutable Operation IR → finalize（SQL・ordered typed args・semantic metadata）→ policy/risk inspection → public review view → application-only execution gate → SQLx adapter → result inspection → transaction outcome。

1つのfinalized recordが、検査したSQLと実際に送るSQL・値を保持する。公開JSONや表示用SQLを読み戻して実行しない。RETURNING、conflict branch、batch、COUNTの変換を含め、最終SQLの確定後に検査する。表示後にSQLを追記する経路を作らない。

Plan/compile/reviewは純粋・決定的に動き、DB接続、live catalog、EXPLAIN、認可provider呼出しを暗黙に行わない。live schema、EXPLAIN、外部認可は別の明示APIで取得する。

通常CRUDも共通gateを通す。検証済みhandleを使う遅延実行では、target/dialect、executor/session、tenant/subject、policy/schema/設定世代、operation全体、期限、現在アプリcontextをdispatch直前に照合する。古いsnapshotを「現在」と再供給するアプリの誤りや、接続先の物理的置換まで自動認証したとは言わない。

初期既定はStrict。未対応・不明・必要context欠落・不鮮明な型/列は、実行可能な低リスクへ変換しない。Goquent旧互換モードを持ち込まない。後日互換/診断専用モードを追加する場合も別仕様で区別する。

### 3.1 所有権と非同期処理

ValidatedOperationは元IRとcontextのprivate snapshotを保持し、executeに所有権を渡す。execute futureを作った時点でhandleは呼出し元に戻らない。未poll、future破棄、取消、期限切れ、mismatch、driver失敗も再試行可能な同一handleを返さない。共有aliasやserdeで複製できるAPIを設けない。

これはRustの所有権を利用したAPI制約であり、DB処理のexactly-onceではない。dispatch後のcancel/timeoutでcommit状態が確定できなければOutcomeUnknownを返し、commandの同ID照合等の明示手段を使う。通常CRUDを自動再送しない。

trusted providerとのawait後には現在context・期限・exact operationを再検査する。アプリcallbackに生のexecutorを渡す迂回経路は共通gateの保証対象に含めない。

### 3.2 値・型・IR

- 型付きValue enum等で整数の幅/符号、bool、UTF-8 text、bytes、UUID、exact decimal、date/timeを区別する。任意Any/Display/Serialize/driver callbackから値を推測しない。
- exact decimalはfloatへ変換せず、precision/scaleとdialectのencode/decode対応を明示。未実装の型はunknown/refuse。
- SQL NULL、値欠落、空文字、0、falseを区別する。nullability unknownはnon-nullに昇格させない。
- filterはAND/OR/NOTの木、table/alias/column identity、演算子、literal/ref、NULL意味論を保持する。
- predicate/value/typeのcanonical化とdriver束縛が異なる意味にならない対応表を持つ。hash単独を一致判定に使わない。
- 動的schema/列を選ぶAPIにも同じruntime検証が必要。型が通ることと実行権限は別である。
- SELECTは明示projectionを基本とする。任意SQL断片、任意function、CTE、subquery、複雑joinは初期Strict保証範囲にしない。

初期のtenant/単一行証明領域は、既知の非NULL signed SQL整数、正しいkey/schema metadata、通常base table、支持した比較演算子に絞る。string tenant等は型・collation等の仕様とテストを追加するまで証明を拡大しない。通常の文字列列は型検証されたデータとして扱える。

## 4. ポリシー・risk・認可

### 4.1 Tenantとschema

tenantは認証・権限確認済みのサーバーコードがApplicationContextへ供給する。HTTP/AI JSONのtenantやsource文字列をそのままtrustedへ昇格させない。constructor自体が外部認証を行うわけではなく、アプリが信頼境界を守る。

自動tenantはcaller WHERE全体を括ってその外側へAND追加する。ORの各枝から抜ける条件にしない。明示tenantが不一致なら上書きせず拒否。UPDATE tenant列は同値でも拒否。INSERT/batchは全候補を検査し、欠落時のみ補充する。soft-deleteの既定filterはtenantを取り除かず、物理DELETEを論理DELETEへ暗黙変換しない。

複合key、nullable unique、alias、join、conflict targetの完全性を扱い、idという列名やLIMIT=1から単一行更新を証明しない。ORは全枝の証明が必要。schema/trigger/view/RLS/生成列/照合順序等の対応外はunknown/refuseとする。

### 4.2 Risk・抑制・認可

riskはlow/medium/high/destructive/blockedの閉じた分類にする。unknown coverage/partial/unsupportedを独立に保持し、表示上lowでもStrict実行の必要証明が欠ければ拒否する。

suppressionとbaselineにはcode、限定scope、reason、owner、expiryを持たせる。構造block、tenant違反、必要なfreshness失敗、検証不完全性は抑制不可。表示の抑制は実行許可にならない。

high/destructiveには、対応可能な操作に限り、アプリが明示登録した外部認可providerのrequest-bound decisionを要求する。reason、PII access reason、manifest Fresh、public JSON、boolean allowをpermitにしない。providerが未設定なら拒否。構造block/未知操作/不正schema/Raw等を外部permitで解除しない。

認可の主体はアプリが認証する。providerはscope/tenant/operation等を見て判断するが、別途契約しない限り外部サービスがSQLや全値をレビューした証明とは言わない。未実装の認可統合を先行PRで宣伝しない。

## 5. 出力・Manifest・AIインターフェース

### 5.1 SQL-visibleと機密境界

| 出力 | 許可する内容 |
| --- | --- |
| 既定CLI/JSON/SARIF/MCP/tracing/Debug/Display | 固定code、risk、coverage、precision、構造count等のpublic view |
| 明示ローカル詳細artifact | SQL、型付き値、table/column名、詳細診断。機密扱いで新規ファイルへ出力 |
| 実行内部 | private IR、SQL、args、executor/context/binding/permit。公開serdeから復元不可 |

既定public viewにはSQL/値/tenant/識別名/path/snippet/自由文reason/任意error/opaque metadataを含めない。local詳細は明示pathとopt-inで新規作成し、上書きを拒否する。CIやMCPへ詳細exportを許可しない。OS別の保護は実際のACL/作成動作を試験し、Unix modeだけでWindowsの保護を保証しない。

公開用に件数や文字数を切り詰めても、CIと実行gateは完全な内部結果に基づく。診断を切った結果を「問題なし」に変えない。driverエラーやsource errorは公開エラーのcauseから漏らさず、明示機密経路と分離する。

### 5.2 Schema/Manifest

schema、policy、generated-code、live DB観測を別々のfingerprint/coverageとして持つ。単なるschema文字列の一致を認可や現在のDB同一性としない。offline consistency、application assertion、live observed、stale、missing、unverifiedを区別する。

Strict実行に必要なschema/policyの権威・対象・世代はアプリ供給の契約で照合し、manifest JSONのfresh文字列だけで満たさない。runtimeの全操作にlive catalog問い合わせを強制しない。必要なfreshnessポリシーと更新方法を設定可能にし、観測後の外部DDL競合は保証の限界として記録する。

source artifactとpublic summaryは別kind/versionとする。外部wireは入力サイズ・深さ・要素数・文字列長を制限し、未知version/重複key/未知field/型不一致/複数documentを拒否する。Goquentとのwire互換はv1要件外。

### 5.3 OperationSpec / MCP

OperationSpec v1は単一model・read-only SELECT、明示fields、有限filter、order、limitに限定する。write、join、aggregate、raw、subquery、任意hintは拒否。tenant refは予約名を通じtrusted application contextから解決し、JSONから値を指定/上書きできない。

compile/validateはDB-free。application-only read APIだけ、同じfinalized recordを共通dispatcherで実行する。MCP compile/reviewには実行権限がない。

MCPはstdio/read-onlyとし、public context、manifest status、review rules、plan/review/compileに限定する。DB CRUD、Raw、migration apply、permit取得/発行、詳細source exportを公開しない。live schema/EXPLAINを暗黙に取得せず、初期MCPはoffline入力から動かす。

## 6. 通常ORMとしての使い勝手

- table/model別のnominal key newtypeで取り違えを減らす。別modelのpredicate/orderを混ぜられない。
- named projectionは選択列と結果型/復号順序を固定する。SELECT *を必須にしない。
- nullable resultはOption<T>。patchはUnchanged / SetNull / Set(value)を分離する。
- generated/readonly/tenant列の更新APIを生成しない。unknown SQL型をAnyで実行可能にしない。
- builder/typed repository/OperationSpecは共通IRへ落とし、3つのplannerを保守しない。
- batch、RETURNING、upsert、joinはcapabilityと保証表を持ち、未対応をDB送信前に拒否。最初から全方言featureを実装したことにはしない。
- 任意RawはStrict保証外。もし後段で人間向けescape hatchを設けるなら明示unsafe相当のAPI・理由・診断を必須とし、AI/MCPから呼べない別契約とする。

## 7. Tx・行数・command

owned TxはQuentがbegin/commit/rollbackを管理し、期待行数、version conflict、RETURNING全結果/復号をcommit前に検査する。borrowed Txは通常操作を許せるが、rollback/commitの所有権を持たないためatomic guarantee APIは拒否する。savepointやrollback-onlyを追加するなら別の明示契約と試験が必要。

MySQLのchanged/matched/upsert件数とPostgresの件数を同一視しない。返された最初の1行を全体のexactly-one証拠にしない。SQL成功、scan成功、Tx commit成功を別結果として保持する。

Command v1はfinite INSERT/UPDATE、typed canonical input、明示operation ID、trusted tenant/target/namespaceを持つ。予約ledger・業務write・結果・audit/outboxは同じowned Txに置く。DDLは利用者が導入し、ライブラリが自動作成/変更しない。

同ID同入力は保存結果の照合、同ID異入力はIdentityConflict。ID/入力は初回送信前にアプリ側で永続保持する。commit応答喪失はOutcomeUnknownとし同ID同入力で照合する。無条件の自動再送、新ID置換、ledger不正をmissingと解釈する処理は禁止。ledgerの削除/保持期限と権限はアプリ責任。

audit/outboxのDBへの書込みまでがatomicity対象。外部配送・API呼出し・callback副作用のexactly-onceは保証しない。

## 8. Migration・性能・静的レビュー

Migrationはschema diff→plan→phase/preflight→明示human apply。列default/index順序/FK/CHECK/namespaceをcapabilityで扱い、unsupportedを安全としない。preflightには対象/plan/観測条件/時刻/期限/可視範囲を結び付ける。Postgresのtransactional DDLとMySQL implicit commit差異を明示し、multi-step完全rollbackを一律保証しない。

性能のPlanは通常QueryPlanと別。非ANALYZE EXPLAINを明示実行するだけで、推定rows/indexを実測や性能改善の証拠としない。request別budgetはdispatch前、実時間と結果消費は観測後に区別する。N+1は疑いとして提示し、診断から業務値やSQL/hashを漏らさない。

Review CLIは最初にplan/OperationSpec/manifestを対象にする。Rust sourceはstableで扱える有限範囲に限り、synの構文一致を型同一性やruntime対応の証明にしない。macro/helper/alias/cfg等で復元できなければpartial/unsupported。外部plan JSONのverdictは再計算してもruntime correspondenceはunverifiedのままにする。

SARIF/diff/baselineは完全head検査の表示補助。変更行だけでgateを決めず、期限付きbaselineもblockや不完全性を弱めない。公開SARIFにpath/snippetを含める別例外はv1で設けない。

## 9. 受入ケース台帳

各caseには入力・前提schema/context・期待診断・dispatch回数・Tx結果・対応方言・precision・未検証範囲を記録する。GoquentのCxxと対応付ける際は意味の一致と相違を記し、同一SQL文字列を要求しない。

| ID | 受入シナリオ | 必須結果 |
| --- | --- | --- |
| QT-C01 | SELECTのPlan/compile/review | DB/provider呼出し0。反復結果は時刻等を除き決定的 |
| QT-C02 | WHEREなしUPDATE/DELETE | non-suppressible blocked、dispatch 0 |
| QT-C03 | tenant一致のみのUPDATE | tenant-wideでありsingle-rowではない |
| QT-C04 | id>0、id IN複数、複合key一部 | broad/unknown、id名から単一行にしない |
| QT-C05 | key=1 OR status=active | 全枝を証明できずbroad/unknown |
| QT-C06 | (tenant=T1 AND key=1) OR key=2 | tenant漏れ拒否、auto時はcaller全体の外側AND |
| QT-C07 | NULL、NOT、空IN、nullable unique | SQL意味を保持。空INは有限のfalseへ正規化し結果を明記、未対応型を安全にしない |
| QT-C08 | JSON tenant、古いfresh claim、reasonのみ | trusted context/permitへ昇格しない |
| QT-C09 | batch内1行だけtenant不一致 | 全batch dispatch 0 |
| QT-C10 | tenant列更新、readonly/generated patch | compileまたはruntime拒否、dispatch 0 |
| QT-C11 | DB A/Bの異なるpolicyとtenant | 変更・共有による漏れなし、Txへ正しいsnapshot継承 |
| QT-C12 | Validate後の値/target/世代/subject変更 | mismatch、dispatch 0、新handle必要 |
| QT-C13 | handle再使用/Clone/serde/future破棄 | compile-fail/API拒否、再利用不能 |
| QT-C14 | 期限境界/cancel/認可error/別request decision | 拒否。最終dispatch直前の変更も検知 |
| QT-C15 | 公開JSON/Debug/error/SARIF/MCPへcanary値 | SQL/識別名/値/reason/source errorの漏れなし、実行復元不可 |
| QT-C16 | 期待行数不一致/楽観version/復号失敗 | owned Tx rollback。borrowed Txにatomic保証なし |
| QT-C17 | command同ID同入力/異入力/同時実行 | replay/IdentityConflictを区別、業務write二重化なし |
| QT-C18 | commit応答喪失/ledger破損/outbox再送 | OutcomeUnknown→同ID照合。不正ledgerを再実行しない |
| QT-C19 | schema/policy/code/live差分/未観測 | 対象別freshnessを表示、offline一致をliveとしない |
| QT-C20 | source macro/helper/cfg/動的query | partial/unsupported、CIで無条件成功にしない |
| QT-C21 | EXPLAIN/観測/budget | 非ANALYZE、明示取得、Planは暗黙取得なし、budget拒否はdispatch 0 |
| QT-C22 | migration未対応/古いpreflight/DDL失敗 | 拒否/限界表示、MySQL完全rollbackを装わない |
| QT-C23 | MCP write/raw/apply/permit要求 | tool不存在または拒否、DB/provider呼出し0 |
| QT-C24 | 不正JSON/未知version/巨大入力 | 有界に拒否、旧receiver/低risk既定へフォールバックしない |

Compile-failでは「別key/modelの混在」「handle再使用/複製」「viewから実行」「非nullableへNULL」を確認する。型検査だけでは期限・tenant・schema・driverの事実を証明できないためruntime拒否テストも必要。

## 10. 実装順・完了判断・運用

- M0: QT-AI-00。仕様をリポジトリへ定着し、空のRust基盤と受入fixtureを作る。
- M1: QT-AI-01〜07と08 PR1。両DBで型付きCRUD→Plan→policy→共通gate→結果、基本reviewを通す。認可未実装のhigh riskは拒否。
- M2: QT-AI-08残り、09〜13。認可、永続command、migration、性能、read-only MCPを拡張。
- M3: QT-AI-14。全体受入と公開準備。実装済み機能だけをv0.x機能表へ載せる。
- PRは依存PRがbase mainへmerged=trueと確認してから次を開始。同一Issue内の前PRも必須。
- 1 work = 1 planned PR。実PR番号は作成後に記録し、PR1等の計画番号と混同しない。
- 各PRの報告はhead SHA、差分、実行した検証、skip/fail、両DBのversion/設定、受入case、残る制限を含める。
- workerが報告しただけでIssue完了/merge済みとしない。Issue全体ACとmain統合を別途確認。
- 実装時に未確定事項が保証やscopeを変えるなら仕様相談と次版へ。版登録と実装起動は別。先行work revision 2はPR1の実装起動だった（運用履歴）。

## 11. 共通worker指示

対象はrecoweft/quentのみ。Goquentは参照資料であり変更しない。base main。下記の指定Issue・planned PR・branchだけを実装する。依存を確認し、未mergeなら着手しない。実行範囲は現在の明示依頼で指定されたplanned PRに限る。

実装開始時はAGENTS.mdと依存固定値を確認し、Issue本文・本仕様・前PRのmain結果を読み直す。公開APIを増やす前に共通IR/gate/出力境界へ接続する。partial/unsupportedを成功へ変えない。通常CRUDとtyped/specの別実行経路を作らない。テストは該当caseと両DBの必要範囲、compile-fail、feature/doc checksを実行する。結果を偽らずskipの理由を報告する。

Issue作成とwork登録は実行ではない。先行実装work 6b291980-36b5-48b1-8e9f-eff4bc3aed65 revision 2はPR1に限定されていた（運用履歴）。公開release、crates.io publish、無関係repo変更は含まない。

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

## QT-AI-04 PR1 output boundary

[Public views v1](public-views-v1.md) adds output-only finite projections of the
current retained record/inspection/preparation and fixed errors/result states.
Historical statements above deferring public views describe their named baseline.
No public JSON or copied display classification is execution authority. Sensitive
getters and the complete private gate remain unchanged; CLI/tracing/export remain
with PR2/owning issues. All 24 full runtime_pending markers remain.

## QT-AI-05 planned PR1 manifest boundary

[Manifest source v1](schema-manifest-v1.md) adds optional DB-free finite source/wire,
independent domain canonical/fingerprint and offline evidence. C19/C24 have bounded
production API positive/negative evidence, with C01/C07/C08/C12/C15 boundary coverage.
Source parsing/equality never supplies live catalog/authorization or trusted facts;
existing Strict/context/validated/atomic gates remain unchanged. Full runtime_pending
stays on all 24 fixture cases. PR2 owns explicit live catalog, drift and CLI verify;
codegen and business authority remain with their owning Issues. Measured commands,
exact head and CI/DB regressions are recorded in the execution/PR report.

QT-AI-06 PR1 source syntax, finite refusals, application role assertions and remaining
typed compiler duties: [OperationSpec v1](operation-spec-v1.md). The initial source
subset has no trusted tenant-ref wire; tenant context is application-only.

## QT-AI-07 planned PR1 code artifacts

[Finite typed Rust generation](typed-codegen-v1.md) adds optional pure codegen,
model-specific keys/columns/predicates/order/projection/results/repositories and
explicit application mapping/context binding through the existing read compiler
and same finalized-record dispatcher. Source metadata, generated Rust and public
summary/diagnostic JSON remain separate untrusted/output-only domains. No patch,
regen workflow, new write/provider, authority restoration or risk relaxation is
introduced. Prior PR absence statements above are historical. All 24 full
runtime_pending markers remain unchanged; actual new-head evidence is reported
separately.

QT-AI-07 PR2: [explicit v2 patch/regeneration boundary](typed-patches-regeneration-v2.md)
adds private model-specific three-state patches for common DB-free preparation and
inspection, plus exact pure prior/proposed/existing byte comparison. It adds no
execution connection or filesystem I/O. Output-only patch summary/diagnostic and
regeneration state/action/count never carry code, diffs, names, values or paths.
V1 generator bytes, SensitiveDetails export restrictions and all runtime_pending
markers remain unchanged.


QT-AI-09 planned PR1: [explicit scoped authorization acquisition](authorization-contract-v1.md) adds
immutable per-DB trusted registration and consumed exact request-bound candidates.
Successful acquisition has no dispatcher/Tx/permit conversion; existing Strict
High/Destructive refusal and all 24 full runtime_pending markers remain.


QT-AI-11 PR1: [offline immutable migration plan](migration-plan-v1.md) adds a separate
Core typed schema/difference API, output-only migration view and protected local
migration detail export. Existing manifest v1/fingerprint and CRUD output formats
are unchanged; no preflight/apply/provider/handle/CLI/MCP execution is added.
See [finite C22-centered evidence](migration-plan-v1-evidence.md). All 24 full
runtime_pending markers and original fixtures remain unchanged.

QT-AI-12 PR1: [explicit finite estimated performance evidence](performance-plan-v1.md) uses the common retained-record gate and native binders. Its application-only SELECT acquisition is non-ANALYZE and Partial; no implicit EXPLAIN, measured-performance/authorization claim, public authority restoration or full runtime_pending completion is added.

## QT-AI-13 PR1 finite MCP boundary

[Offline read-only MCP](mcp-v1.md) implements the stdio 2025-11-25 finite profile,
fixed public resources/tools and application-supplied mapping/current library path.
Ordinary CLI typed compilation remains unavailable without registration. Protocol
correlation is isolated to response top-level id; business payload boundaries remain.
No DB/provider/Tx/permit/live/execution capability or full runtime_pending clearance.
