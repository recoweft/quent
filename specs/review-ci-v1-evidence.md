# QT-AI-08 planned PR3: measured evidence

Work648988f2-5b83-469f-b424-d6f40fc3d507 revision1, public execution
1bc43fc4-2897-4db0-a912-ef1b221b8efc. Measurements2026-10-26 JST.
This records finite subcases; all24 full runtime_pending fixture entries remain
byte-identical. Exact PR/head and hosted conclusions belong to the fixed execution
report. A report receipt is distinct from user acceptance and PR creation.

## Identity, prerequisites and capability checks

Actual cwd /home/murai/github/quent, origin SSH recoweft/quent, WSL2 kernel
6.18.54.1-microsoft-standard-WSL2, Ubuntu24.04.5, rustc1.94.0. Repository and checked
ancestors had no applicable AGENTS.md; .agents/.codex were empty. Initial worktree
clean, one registered worktree. No open repository PRs or Issue9 comments were
returned by approved API reads. This is bounded conflict checking, not a proof
about external workers/controller leases. Supplied work/revision/execution identity
was retained; no start_work or alternate work selection occurred.

Fetched origin/main af7ca64e51ed5943edfde715b35a5457afcd3588. GitHub PR34 merged=true,
head e3ae5420e2d794bb81dc767e65c43b4c91f02885 and merge af7ca64; local ancestry
checked. First-parent main history retains PR33 and dependency PR18/19,25/26,27/28,
29/30. Requested feature/qt-ai-08-review-ci was created from this main. Prior-head
controller measurements were treated as prerequisites, not new-head test evidence.

Sandboxed DNS/SSH, gh auth-status and Docker socket checks failed; these did not
establish invalid credentials or broken host. Approved diagnostics succeeded:
SSH main ls-remote, worker identity recoweft, repository push/admin permission,
specified branch dry-run push, Docker29.8.0 and both pinned image digests locally
present. Git metadata writes require approved execution because .git is read-only
in the sandbox. No global policy/network/DNS/firewall/WSL changes were made.
Local Windows processes/build/SDK were not used as proof; native hosted Windows
runs are separately checked on the actual head. Windows live DB/ACL success remains
unverified/unsupported.

## Local finite verification

| Command / selection | Observed result |
| --- | --- |
| `cargo build --workspace --locked --offline` and `--all-features` | passed |
| `cargo test --workspace --all-features --locked --offline --no-run` | passed |
| `cargo test --workspace --locked --offline` |85 passed,0 failed/ignored/filtered |
| `cargo test --workspace --all-features --locked --offline -- --skip live_ --test-threads=1` |201 passed,0 failed/ignored,10 substring exclusions |
| Three incidental substring complements: Core exact-context, manifest live-claim nonpromotion, CLI live-config |1 passed each (actual named selections) |
| `bash scripts/test-dispatch.sh cargo test --workspace --all-features --locked --offline -- --test-threads=1` |211 passed,0 failed/ignored/filtered; both DB suites and retained regressions |
| `bash -x scripts/check-features.sh --offline` |262 cargo checks and retained standalone CLI tests passed |
| All30 documented facade and8 CLI feature combinations, test then warnings-denied all-target Clippy |76 sequential commands passed |
| `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings` |passed |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked --offline` |passed |
| `cargo fmt --all -- --check`, `git diff --check`, new schema JSON syntax |passed |
| Normal Core/default facade/default CLI and standalone review-ci dependency trees |Core empty; defaults Core-only; review-ci driver/runtime/source-parser free; lock unchanged |

Final compiler/CLI builds and runs were sequential. Shared compiler assertion
checks203 negative programs and11 successful controls (retains189/10). Fresh
production-generated consumers retain76 negatives/eight controls across six
invocations. Two new review-ci compile-fail doctests are separately counted in
workspace test totals. The new CLI tests use the actual product executable.

Development compile failures in the new test fixture (glob-imported Value and
missing Draft.freeze) were corrected before final runs. Sandboxed CLI tests had
15 passes/three failures from loopback-bind PermissionDenied; approved rerun passed
all23 CLI tests for review-ci,source-review. These were environment failures, not
product non-success or skipped accepted tests. A first manifest incidental
complement filter selected zero tests; the actual named complement was run
separately. Zero-selected filters are not counted as evidence. No approval denial
occurred during these checks.

## Finite product conclusions and limits

New kind/version/grammar/exits, source envelope, exact diff semantics, fixed SARIF
allowlist, baseline bounds and all non-suppressible codes are documented in
[review-ci contract](review-ci-v1.md). Independent empty JSON/pretty/SARIF goldens
pin exact stdout/LF/stderr/exit3. Nonempty tests retain full independently checked
old review results; SARIF is parsed and recursively checks its permitted members.
URI/markdown/control-bearing claim canaries and sensitive names/labels/SQL/value/
reason/owner/hash never enter any new public stream/Debug/Display/error/tracing.

Exact supplied source bytes+format+target/context/scope and fixed analysis version
are required for unique unchanged classification. Public summary/code/count or
claimed identical fingerprints cannot hide distinct content. Multiplicity is
preserved; ambiguous duplicates and unavailable application facts stay unknown.
Base removed display never certifies an empty head. All original findings remain
non-suppressible. Only the new external-claims notice can be hidden with strict
whole-input match, reason/owner and finite exclusive expiry; full original claims
remain unverified. Deterministic explicit time tests cover before/at/after expiry,
unknown time, overflow and far-future non-hiding.

Full head results precede diff and hiding. Explicit inputs processed, changed-only,
missing head and unverified Git head/runtime/authority are distinct. No success
exit exists; full Blocked5/Unsupported4/Unknown-Partial3 or presentation Budget2.
Public detail budget64 does not remove complete private diagnostics (cap256):
exhaustion is explicit non-success. No CI/SARIF/JSON output enters execution gates.
No new DB/provider/write/Tx/live/network/upload/export/cargo/build.rs/proc-macro/
Git/discovery/clock capability exists in the pure library. Clock is explicit in
library and collected only at the CLI boundary.

Every new file position has actual byte/UTF8/grammar/budget/duplicate tests,
Unix writerless FIFO/directory/device/final symlink and opened-object replacement
races with kill+reap child watchdogs; native Windows device/reparse/replacement
checks are in hosted CI. Same opened handle is not a filesystem snapshot or
OS-wide response deadline. TCP and tracing detectors have actual positive controls.
The historical live DB regressions still test common Strict/CRUD/validated/atomic/
catalog/CLI/codegen/patch/regen; finite review adds no execution permit or semantics.


## Disposable database measurements and cleanup

Both newly created pinned images ran PostgreSQL16.14 and MySQL8.4.8. PostgreSQL:
UTC/read committed/UTF8/en_US.utf8. MySQL:+00:00/REPEATABLE-READ/utf8mb4/utf8mb4_bin,
ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,
ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION; retained FOUND_ROWS adapter.
Only script-created dedicated quent_dispatch_test/manifest_other/
quent_manifest_other namespaces, quent_test/hidden reader roles and generated
fixtures were used. Script cleanup removed both containers and their anonymous
volumes and verified absence. No existing database was consumed. Approved full
execution succeeded; no live test was skipped or accepted from an environment
absence. The211 full total includes the prior196 and15 new tests/doctests.

Full workspace includes seven actual live test functions spanning both DB adapters,
existing output canaries/compiled/generated reads/patch and catalog/CLI workflows.
These are retained finite measurements, not general authenticated physical-server,
TLS, external DDL race or network completion-fault guarantees. Windows live DB,
Windows detailed-export ACL success and OS-wide I/O deadlines remain unverified.
All24 full runtime_pending markers remain unchanged. Exact-head Ubuntu/Windows
hosted results are separately reported after push/PR creation; this document does
not treat workflow presence or prior-head hosted results as passing evidence.

Initial hosted head e459aa7ad0d19c12481eac7acd9849471dc81354 exposed a missing
.gitattributes rule for new golden files on Windows checkout. Two new CLI exact-byte
assertions failed (393-byte LF stdout versus394-byte CRLF expectation); extracted
log arrays were otherwise byte-identical after CRLF normalization. The new fixture
directory is now explicitly LF, matching existing fixture policy. No product input,
output or acceptance oracle was weakened. Failed/cancelled jobs on this initial
head are not counted as passing; final exact-head hosted runs are checked separately.
