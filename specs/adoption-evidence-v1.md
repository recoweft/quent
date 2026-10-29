# QT-AI-14 PR2 documentation verification

Work `b3e73336-c4e5-4aa1-b326-dd8fcbc428ca`, revision 1, public execution
`34a11a3a-88e5-4b65-9a41-ee7d341cfb08`. These are new local measurements on
2026-10-29, not PR51 receipts. The exact committed source head, actual PR URL and
final-head hosted CI results are recorded in the separate PR/RelayWeft report;
this file does not predict hosted results.

## Identity, base and scope

Checked actual cwd `/home/murai/github/quent`, Ubuntu 24.04.5, WSL2 kernel
6.18.54.1-microsoft-standard-WSL2, local AGENTS.md, origin
`git@github.com:recoweft/quent.git`, branch `feature/qt-ai-14-release-docs` and
HEAD `e52f23f0c8815c72a050571a54645c8b683f284a`. The requested branch and seven
uncommitted documentation files already existed at entry; they were preserved
and reviewed. No checkout/reset discarded them. GitHub API identity and SSH
authentication both identified `recoweft`; repository API reported push/admin
permission. SSH's authenticated/no-shell exit1 is expected, not a push test.
Actual push/PR creation outcomes belong in the final report.

Fetched real origin/main at that same SHA. Read current Issue1/15 bodies and
comments (both comment arrays empty). Rechecked actual PR17/19/21/24/26/28/30/32/
35/38/41/47/49/50/51: all merged, base main, exact supplied merge SHAs. Separate
`git merge-base --is-ancestor` checks passed for all15 against fetched origin/main.
This verifies merged prerequisites, rather than treating controller receipts as
new measurements.

Read the local Goquent committed contract case table at
`c51e084d0d530cae3eb22ad2b0686e47c6b79f0c` and checked its clean worktree.
Goquent was not modified, fetched, built or tested. The [comparison](goquent-case-comparison-v1.md)
remains documentary only. Remote-current, Go runtime and wire compatibility are
unverified.

Reviewed the new entry guides against integrated/domain contracts, the original
and integration case registers, feature manifests, existing public example,
candidate review/common dispatch/native binding sources, CLI/MCP and CI scripts.
No new Rust snippet, production/API/output path or docs mirror test was added.
Commands below used the locked cached dependency set with cargo `--offline`.

## Local commands and observed results

| Command / group | Result |
| --- | --- |
| `rustc --version`, `cargo --version` | Both1.94.0; edition2024/toolchain and workspace pins retained |
| `cargo build --workspace --locked --offline` | exit0 |
| `cargo test --workspace --locked --offline` | 114 passed,0 failed/ignored/filtered; feature-disabled targets also report0 selected |
| README `tenant_review`, `codegen,review` | exit0, six public JSON lines; exact regeneration NoChange, inspection-only patch3, Unknown patch/read/manifest reviews, SqlPlanned High migration1/live Unverified |
| Explicit PG and MySQL `cargo check` with `runtime-tokio,tls-rustls` | Both exit0; build evidence, not TLS handshake evidence |
| Core/default facade/default CLI normal-edge trees | Core0 edges; facade and CLI only Core |
| Facade and CLI `mcp`-only normal-edge trees | No SQLx or Tokio driver/runtime edges |
| `cargo fmt --all -- --check` | exit0 |
| `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings` | exit0 |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked --offline` | exit0 |
| CLI review plan SELECT / Rust syntax fixture | exits3/3, Unknown/Partial respectively |
| CLI offline manifest-v1 self-comparison / codegen-v2 self-comparison | exits3/0, Unknown/OfflineConsistent; both live Unverified |
| CLI empty review-ci inputs/baseline | exit3, Incomplete |
| CLI `mcp`-only tests | 9 passed,0 failed/ignored/filtered, exit0 |
| Facade `mcp_stdio` with `mcp` only, approved execution | 10 passed,0 failed/ignored/filtered, exit0 |
| All-feature CLI build and workspace test `--no-run` | Both exit0 before starting DB fixture lifetime |
| Dedicated both-DB full workspace/all-feature suite | 290 passed,0 failed/ignored/filtered, exit0; cleanup/absence confirmed |
| Generated consumer within full suite | 76 diagnostic-checked refusals +8 positive controls in6 invocations; freshly generated artifacts |
| `owned_api` within full suite | 425 diagnostic-checked refusals +25 compiling controls; all assertions passed |

The CLI commands use the exact checked-in paths/grammar in
[reproduction](reproduction-v1.md); nonzero analysis exits are expected results.
The standalone MCP test's initial sandboxed run stopped after8 groups at its
loopback detector bind (PermissionDenied), exit101. It was not acceptance. The
approved run above completed all10; no assertion was changed. The all-feature full
suite independently completed its MCP groups. Compiler/consumer control counts
are inside harness tests and must not be added again to290 as test functions.

## Dedicated DB run and finite responsibilities

Executed the unchanged `scripts/test-dispatch.sh` with
`cargo test --workspace --all-features --locked --offline -- --test-threads=1 --nocapture`.
Docker29.8.0 and the script's two digest-pinned images were checked before use.
The script created only new unique loopback containers/anonymous volumes and its
dedicated fixture roles/namespaces. PostgreSQL16.14 reported UTC/read committed/
UTF8/en_US.utf8. MySQL8.4.8 reported REPEATABLE-READ,+00:00,utf8mb4/utf8mb4_bin and
ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,
ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION. InnoDB and pinned driver
FOUND_ROWS behavior remained under the existing tests. No production/DATABASE_URL,
global host/DB settings or real credentials were used or saved to public artifacts.
Exit0 and `Dedicated containers and their volumes removed and absence verified`
were both observed. No failed DB run or remaining owned resource occurred.

The full suite reran the [case-to-source register](integration-acceptance-v1.md),
including A01–A23 (36 conceptual subcases per engine), G01–G04 per engine and
seven conceptual M subcases per dialect. These47 concepts are distinct from
harness totals. Per engine, the tenant trace asserted provider19; main business8;
ledger SELECT8/reserve3/result-save2; owned BEGIN10/ACK10, COMMIT5/ACK5,
ROLLBACK5/ACK5. Three real protocol sessions had business3 and client commit ACK1;
fresh reconciliation lookup3/business0 with begin3/commit ACK3. Borrowed read1/
caller rollback1 remained separate. Generated patch inspection submitted0 and
generated read1 retained the same record, with unchanged independent observer data.
Setup/connection/observer traffic is excluded from those business counts.

Retained QT-C02–C10 policy/native refusals, C11–C16 ownership/current/expiry/result/
canary controls, C17–C18 replay/conflict/corrupt-ledger/real response-loss,
C19 catalog, C20 source review, C21 non-ANALYZE estimates/request budgets,
C22 preflight/apply and C23–C24 MCP/bounded parsers all passed their finite tests.
PR45 cancellation/independent absence/ACK controls and migration assertions were
unchanged. Unknown remains distinct from rollback ACK; no external delivery
exactly-once or unobserved-path budget coverage follows. Raw benchmark timings are
local fixture observations, not evidence of an improvement or a release criterion.

## Environment approvals and remaining checks

Sandboxed GitHub DNS/API access failed; the auth tool's initial invalid-token
message was not proof of bad credentials. Approved read-only API/SSH diagnostics
succeeded, followed by approved fetch/Issue/dependency reads. Sandboxed Docker
socket access was denied; approved daemon/image diagnostics succeeded, followed
by the approved dedicated fixture run. The sandbox MCP bind failure and approved
success are recorded above. No approval was denied or bypassed; no global network,
DNS/firewall/WSL/Codex policy was changed.

The unchanged312-check feature script and all94 hosted CI jobs are verified
separately at the final source head, not claimed from a local repeat of that
matrix. No new Windows-local program/dependency/live DB/ACL test was attempted;
the prior pre-test dependency/unsigned-script-policy stop remains unverified.
Hosted Windows DB-free results cannot clear that limitation. All24 original
runtime_pending markers, fixture expectations and loader vocabulary are unchanged.
Broader unsupported behavior and separate release/tag/publish/version/license
authorization remain in the [publication holds](release-checklist-v1.md).
