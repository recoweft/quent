# QT-AI-09 planned PR1 finite measurements

Work 436b76c5-9353-41b7-9a04-5c8eb91e880c revision 1, public execution
97abd313-59da-41c6-8c87-4eedf75518a6. Measured 2026-10-26 JST.
Exact pushed head, PR and new-head hosted results are recorded in the fixed
execution report; prior-head reports are prerequisites, not new-head evidence.
Report receipt is not user acceptance. All 24 full runtime_pending entries remain.

Actual cwd /home/murai/github/quent; WSL distribution Ubuntu, Ubuntu 24.04.5,
WSL2 kernel 6.18.54.1-microsoft-standard-WSL2, Rust 1.94.0. Origin is the requested
SSH recoweft/quent remote. Repository/checked ancestors had no applicable AGENTS.md;
.agents/.codex had no files. Initial worktree was clean, one registered worktree.
Approved API read returned no open repository PRs. This is bounded conflict checking,
not verification of external workers or controller leases. Supplied work/revision/
execution and actual cwd/environment match. No start_work/alternate task selection.

Fetched main b66308db78705322d2a7efdeaf68c569a2d2dc3e. PR35 API state MERGED and
merge SHA match; local ancestry includes head3e1907b and first-parent PR20/21,
22/23/24 and25/26 merges. Requested branch was created from that main. Issue10
was read through the approved worker API before implementation. Issue36 is excluded.

Sandboxed Git DNS, gh auth-status and Docker socket diagnostics failed. These
were environment restrictions, not evidence of invalid credentials/broken host.
Approved bounded diagnostics succeeded: main ls-remote, actual worker recoweft,
repository push/admin permissions, specified-branch SSH push dry-run, Docker29.8.0
and both pinned images already cached. PR creation is verified by the actual
subsequent operation/report, not inferred from the Issue connection or dry-run.
Git metadata mutations use approval because .git is sandbox read-only. No approval
was denied; no settings/network/DNS/firewall/WSL policy was changed. Local Windows
program launch/SDK/build was not tested; hosted OS results are separately reported.

| Final local command/selection | Observed result |
| --- | --- |
| `cargo test --workspace --locked --offline` |93 passed,0 failed/ignored/filtered (85 retained+8) |
| `cargo test --workspace --all-features --locked --offline -- --skip live_ --test-threads=1` |210 passed,0 failed/ignored,10 substring exclusions (201 retained+9) |
| Exact incidental complement filters: Core context, manifest live-claim, CLI detecting config |1 passed each; no zero-selected evidence counted |
| `bash scripts/test-dispatch.sh cargo test --workspace --all-features --locked --offline -- --test-threads=1` |220 passed,0 failed/ignored/filtered (211 retained+9); both DBs; cleanup verified |
| Shared real compiler harness |243 diagnostic-checked negatives+12 compiling controls; retains203+11 |
| Fresh production-generated consumer harness |76 checked negatives+8 controls in6 invocations, retained |
| `bash -x scripts/check-features.sh --offline` |262 cargo checks and existing standalone CLI test selections passed |
| Existing30 facade and8 CLI combinations, each test then warning-denied all-target Clippy |76 sequential commands passed; new default/std-only authorization participates in every facade combination |
| Default/all-feature workspace build, `--locked --offline` |passed |
| Workspace/all-target/all-feature Clippy with `-D warnings` |passed |
| All-feature/no-deps rustdoc with `RUSTDOCFLAGS=-D warnings` |passed |
| `cargo fmt --all -- --check`, `git diff --check` |passed |
| Normal Core/default facade/default CLI/review-ci trees |Core empty, defaults local Core only, review-ci no driver/runtime or source-review parser; lock/pins unchanged |

Final shared compiler and CLI builds/runs were sequential. Early development
Clippy manual-async/useless-vec findings were corrected; they are not final passes.
No new dependency/feature/lock/toolchain/edition/publish setting or CI job changes.
Original fixture bytes and their documentary expectations remain unchanged.

[Contract and support matrix](authorization-contract-v1.md) distinguish actual
production acquisition from absent permit dispatch. Nine new all-feature library
tests include eight default tests and one tracing test with an emitting detector
control. Candidate creation/Plan/preparation/typed compile/review leave a registered
fake counter at0; explicit acquisition raises it. Before/at/after expiry uses the
same private production checker with deterministic Instants. Public acquisition
checks expired entry and expiry across an actually invoked provider await. Tests
also cover deny/error/provider-reported cancel/timeout, unpolled0/polled1-drop,
saved cross-request decision mismatch, full immutable context/provider separation,
retained input differences and private/nonClone/nonCopy/serde/restoration boundaries.
Borrow checking forbids replacing the borrowed current snapshot while the future
remains live; external mutable stores are not checked by that proof.

Both real live driver suites acquire successfully from a counted in-process fake,
then observe High/Destructive business sends0 through the unchanged production
Strict gate. Ordinary Low CRUD sends1 and existing owned atomic Low succeeds,
with provider count unchanged at1. Existing validated/atomic/borrowed/catalog/CLI/
FIFO/review/source/CI/codegen/patch/regen and driver-output regressions pass.
These are finite C08/C11–15 evidence and supporting C01/C02/C06/C10/C16/C19/C23
boundaries, not full acceptance of those cases. No acquisition DB/Tx records are
fabricated. All24 full runtime_pending markers remain byte-identical.

Disposable servers used the unchanged script digest pins: PG16.14 UTC/read
committed/UTF8/en_US.utf8; MySQL8.4.8 +00:00/REPEATABLE-READ/utf8mb4/utf8mb4_bin,
ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,
ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION and existing FOUND_ROWS.
Only new dedicated containers/namespaces/catalogs/roles/fixtures were used. Script
removed both containers and their anonymous volumes and verified absence after
final full run. No existing database or global Docker prune was used.

Unverified/PR2: final runtime request/session/current operation/freshness binding,
external current-store changes during provider await, actual permit dispatch/Tx
consumption, automatic timeout/cancellation detection, external service protocol/
authentication/review/cancellation/exactly-once, physical routing authentication,
TLS/network completion faults, concurrent DDL, Windows live DB/SDK/local process
and detailed-export ACL success. Adapter-response Timeout/Cancelled are implemented
classifications, not automatic detectors. Windows protected export remains
UnsupportedOs; CI/MCP details refusal remains. Hosted Ubuntu/Windows exact-head
jobs are checked separately after push; workflow presence is not a pass or proof
of branch protection. No merge/Issue closure/release/publish occurs in this PR.
