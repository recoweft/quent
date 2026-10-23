# QT-AI-00 planned PR1 bootstrap

本PRは仕様・空workspace・依存互換・CIに限定する。PR2のfixture、記録器、runtime機能は追加しない。
Issue #1の旧待機指示は実装依頼revision 2で訂正された。Issue全体はPR2を含むため閉じない。

## 責務と信頼境界

| crate | 将来の責務 | PR1の実体 |
| --- | --- | --- |
| quent-core | private immutable IR、型付き値、schema/policy、finalize、planner、診断、public view | 依存なしの空library |
| quent | typed facade、owned/borrowed Tx、共通private dispatcher、SQLx adapter | 空libraryとoptional依存feature |
| quent-cli | review/manifest/codegen/migration/MCP。coreと同じvalidator/planner | 固定未実装codeで失敗するbinary |

builder/repository/OperationSpecは同じIRへ収束し、finalize後のSQL/ordered argsを共通gateが検査・実行する。
public JSONや表示SQLを実行元にしない。Coreのplan/reviewはDB/runtime/providerへ依存しない。
private snapshotを保持する消費型handle、await後の再照合、公開出力と機密artifactの分離は
今後の必須契約であり、この空workspaceで実装・検証された保証ではない。

## 固定値と選定根拠

- stable Rust 1.94.0、edition 2024、resolver 3。toolchainとrust-versionを固定。
- SQLx =0.9.0のpackage metadataがrust-version 1.94.0を要求する。
  この選択済み依存集合に対する最低対応版を1.94とし、同版で両driver/Tokio/TLSをコンパイルした。
  DBなしcore単体の歴史的最低版を主張するものではない。
- Tokio =1.54.1（metadataのrust-version 1.85）。SQLx/Tokioともdefault features無効。
- `postgres` / `mysql`はdriver依存を個別有効化。`runtime-tokio`と`tls-rustls`は明示選択。
  TLSはSQLx `tls-rustls-ring-webpki`、rustls 0.23.46 / ring 0.17.14 / webpki-roots 1.0.11。
  接続API未実装なのでTLS handshake、証明書拒否、実DB encode/decodeは未検証。
- Cargo.lockは全推移依存を固定する。CIは--lockedで実行。
- 全crate publish=false。crate名/商標/公開可否は未検証。

## 検証と限界

Ubuntu 24.04.5 / WSL2、Rust/Cargo 1.94.0で実行:

- cargo build/test --workspace --locked --offline: 成功。unit/docテストは0件。
- cargo fmt --all -- --check: 成功。
- cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings: 成功。
- RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked --offline: 成功。
- postgres,runtime-tokio,tls-rustlsとmysql,runtime-tokio,tls-rustlsの個別cargo check: 成功。
- cargo tree: coreに外部依存なし。既定workspaceにもSQLx/Tokioなし。

CIはUbuntu/Windowsで既定workspaceと全feature lint/doc、Ubuntuで各driverの互換確認を行う。
実行結果はPR checksで別途確認する。CI設定の存在だけでは実行成功を示さない。
PostgreSQL/MySQLの実サーバー・image/version/設定は未使用・未選定。
実DBintegrationとfixtureは後続PRで導入する。compile-failは対象API未実装のため未実行。
QT-C01〜QT-C24のdispatch/Tx結果、canary拒否、機密artifactのACL等も未検証。

Windows 11 (10.0.26300.9711) / Rust 1.94.0 MSVCでCore/CLI build成功。
最初のoffline試行はWindows側indexの固定Tokio未収録で失敗し、オンラインindex取得後は
UNC上のincremental lock作成で失敗した。最終的に同じworkspaceで
`CARGO_INCREMENTAL=0 cargo build -p quent-core -p quent-cli --locked --offline --target-dir target/windows-check`
が成功した。Windows全feature/実行動作はCIで別途確認する。
UbuntuでRust 1.95.0の全feature cargo checkも成功。CLIのstdoutは空、固定未実装codeで失敗する。

環境確認: cwd、Ubuntu/WSL、remote、local/remote main SHA、Issue #1本文、GitHub repo push権限、
指定branch push --dry-runを確認。適用AGENTS.mdは祖先/作業ツリー内に見つからなかった。
sandbox内GitHub DNS/Windows interopは失敗。承認付き診断は両方成功し、依存取得と
Windows buildも個別承認付きで実施した。remote mutationは別途指定branchのpush/PRのみ行う。
