Current cancellation controls and historical PR43 failure audit:
[QT-MAINT-01 PR1 evidence](authorized-cancel-v1-evidence.md). The measurements
below describe their original PR; the maintenance evidence supersedes cancellation
fixture ordering and finite ordinary UPDATE completion-loss coverage.

# QT-AI-09 planned PR2 finite measurements

Work 667df304-29e2-4970-a39f-870b679cd312 revision 1, public execution
3c0617cf-7c1b-479b-b258-c2e6947bec44. Measured 2026-10-26 JST.
Exact pushed head/PR/new-head hosted jobs are in the fixed RelayWeft report.
A report receipt is not user acceptance. All 24 full runtime_pending markers remain.

## Identity and environment

Actual cwd /home/murai/github/quent, Ubuntu 24.04.5 / WSL2 kernel
6.18.54.1-microsoft-standard-WSL2, rustc1.94.0; specified SSH origin recoweft/quent
and branch feature/qt-ai-09-authorization-gate. No applicable ancestor/repo
AGENTS.md or nested crate AGENTS.md found; .agents/.codex search returned no files.
One registered worktree and no open repository PRs at the initial API check.
This bounded check is not verification of external worker/controller leases.
Five dirty/untracked implementation files predated this execution; controller
answer72d33655-31ac-408a-b839-ad151f72f5b1 explicitly authorized inheriting them
for review/fix/commit. No user work was reset or discarded. No task selection,
start_work, subagent, merge/Issue closure/release/publish was performed.

Approved worker API identified recoweft, repository push/admin permissions; actual
specified-branch SSH push dry-run succeeded. This uses worker authentication,
not an Issue PAT. PR creation is independently recorded after the actual operation.
Sandbox Git DNS failed; approved ls-remote succeeded with main c86464c67277267b84d794964b0d95b6ca363ac3.
Local main ref was stale at initial commit bc7fe02: the initial question's claim of
local-main ancestry success was incorrect and explicitly corrected. Approved fetch
and checked exit statuses verified FETCH_HEAD equals starting branch HEAD c86464c
and includes every merge of PR20/21/22/23/24/25/26/37. API also returned MERGED for
all eight, including PR37 merge c86464c. Issue1 integrated specification was read
with the worker API; Issue10 supplied contents/contracts were reviewed.
Issue36 schema/head-coverage changes are excluded.

Sandbox Docker socket permission failed. Approved bounded diagnostic succeeded:
Docker29.8.0 and both exact digest images cached. Sandbox DB-free CLI positive
controls could not bind loopback (three EPERM failures); approved temporary
127.0.0.1 bind succeeded, and the affected local checks then use approved execution.
No DNS/firewall/network/global Codex/WSL settings changed. No approval was denied.
Local Windows launch/SDK/DB/export ACL was not tested; hosted Windows is separate.

## Finite production observations

Both production dialects use `execution/tests/authorized.rs`; the fake's actual
`authorize` verifies runtime binding, records SQL allocation/full args/metadata,
and independently counts invocations and generated responses. The final production
recorder counts business submissions and Tx events, not network packets. Setup,
connect, begin, observer reads and business send are deliberately separate.

| Case IDs / experiment | Observations and boundary |
| --- | --- |
| C01/C08/C23 plan/compile/review/manifest/generated/Spec regression | no new provider or execution API; ordinary Low needs no provider; PR1 registered fake has a real invocation positive control |
| C02/C06/C10/C19 eligibility | missing WHERE, tenant mismatch/update, INSERT, Partial text predicate, wrong native/NULL/schema, protected/PII/required-filter, version and required freshness refuse before fake invocation and business send (counter0) |
| C11/C12/C14 identity/content | independently supplied target/route/subject/tenant/settings/policy/schema content/identity/generation, same-label new registration, request/session/adapter mismatch refuse with business0; wrong binding at acquisition also provider0 |
| C12/C14 provider suspension | actual pending provider polled and resumed; changed snapshot supplied separately to dispatch; mismatch business0; saved different-invocation decision refuses |
| C13/C14 lifetime | unpolled acquisition provider0; polled pending timeout provider1/response0/business0; unpolled authorized dispatch business0; carriers not returned/retried |
| C14 expiry | real elapsed acquisition/provider/final-boundary/begin expiry, business0; owned begin refusal acknowledges rollback; test-only exact final now==deadline rejects using production checker |
| C11/C12 finite allow controls | direct Broad/FiniteBound High UPDATE and Destructive DELETE send1; provider-inspected SQL allocation, ordered typed args/native NULL and all metadata equal final submission |
| C16 owned | begin/begun/send/decoded/expectation/commit/committed; mismatched expectation rolls back an actually different value, independent observer sees prior committed value |
| C11/C16 borrowed | same-record send1, Uncommitted; external rollback; session/other/new adapter permit refuses; borrowed atomic Unsupported/NotStarted, completion0 |
| C13/C14 real DB wait | both direct/owned authorized UPDATE observed in actual PG/MySQL server lock wait before drop/runtime timeout/disconnect; provider1/send1; quarantine; no commit or confirmed rollback inferred from drop; disconnect OutcomeUnknown |
| C15 output | new constant Debug/fixed errors/source=None/public diagnostics, actual lifetime tracing capture plus emitting detector control; existing driver-notice positive control and nested-error/CLI/Windows export refusal regressions retained |

Caller-supplied current snapshot and immutable borrows do not authenticate an
external context store, routing change, concurrent DDL or adapter interior state.
The library does not invent live freshness, implement automatic timeout, cancel
remote provider side effects, or guarantee exactly-once. Owned authorized
RETURNING/version/all-row decode extension is deliberately Unsupported; existing
ordinary all-row/version/RETURNING decode and rollback regressions are retained.

## Final command results

| Final local command / selection | Observed result |
| --- | --- |
| `cargo test --workspace --locked --offline` | 93 passed,0 failed/ignored/filtered |
| `cargo test --workspace --all-features --locked --offline -- --skip live_ --test-threads=1` | 210 passed,0 failed/ignored;12 substring exclusions (two new actual live suites) |
| Exact incidental complements: Core context; manifest public/live-claim; CLI live-config detecting control | 1 passed each;0-selected mistaken filter excluded from evidence |
| `bash scripts/test-dispatch.sh cargo test --workspace --all-features --locked --offline -- --test-threads=1 --nocapture` | 222 passed,0 failed/ignored/filtered (220 retained +2 new live suites); cleanup absence verified |
| Shared real compiler harness | 272 diagnostic-checked negatives +14 compiling controls (243+12 retained) |
| Fresh generated-consumer harness | retained76 negatives +8 controls /6 invocations |
| `bash -x scripts/check-features.sh --offline` |262 cargo checks and existing standalone CLI tests passed |
| Existing30 facade +8 CLI combinations, each test then all-target warning-denied Clippy |76 sequential commands passed |
| Default/all-feature workspace build |passed, locked/offline |
| All-target/all-feature workspace Clippy `-D warnings`; rustdoc `RUSTDOCFLAGS=-Dwarnings` |passed after final test fixes |
| `cargo fmt --all -- --check`; `git diff --check` |passed |
| Core/default facade/default CLI normal dependency trees |Core empty;default facade/CLI only local Core;no new dependencies/features/lock/pins |

Final compiler harness/CLI build-run commands were sequential. Development failures
were a missing Result unwrap, a test local-name shadow and an incorrect expectation
that composite-key IN would be FiniteBound. They were corrected without changing
scope analysis: the finite positive uses supported full-key tuple OR. A subsequent
full222 and final Clippy/rustdoc passed. Sandbox CLI listener refusals are environment
restrictions, not product passes; the approved full/DB-free runs test those controls.
No zero-selected command or earlier-head CI is counted as a successful test case.

Final disposable servers retain the unchanged digest pins in test-dispatch.sh:
PG16.14 UTC/read committed/UTF8/en_US.utf8; MySQL8.4.8 +00:00/REPEATABLE-READ/
utf8mb4/utf8mb4_bin, ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,
NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION, existing FOUND_ROWS.
Only new dedicated containers/roles/catalogs/namespaces were used. Successful and
failed full runs removed both containers and anonymous volumes and checked absence;
no existing DB or global Docker prune was used. TLS handshake is not established.

Hosted Ubuntu/Windows and both workflow events are checked on the actual pushed
head separately in the RelayWeft report. Workflow presence alone is not a pass or
branch-protection evidence. Local Windows process/SDK/build/live DB/detailed-export
ACL, real external provider authentication/cancellation, physical routing, concurrent
DDL and real lost-completion-reply behavior remain unverified. Required live
freshness, authorized RETURNING/version/batch/upsert/join/new native are Unsupported,
not partially claimed executable support. All full24 runtime_pending entries and
original fixture bytes remain unchanged.
