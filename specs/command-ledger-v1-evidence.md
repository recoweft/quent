# QT-AI-10 planned PR1 finite measurements

Work 3b4894a1-dc54-4a87-9e61-fa959a1f6aad revision1, public execution
75bdb954-c6f8-4c79-bf2d-dfe9bbb82bf5. WSL measurements 2026-10-26/27 JST.
The fixed RelayWeft report records the actual pushed head, PR URL and hosted jobs;
a PR body is not a report receipt or acceptance. All 24 full runtime_pending markers
and original fixtures remain unchanged.

## Identity, prerequisite and approval observations

Actual cwd /home/murai/github/quent, Ubuntu24.04.5 / WSL2 kernel
6.18.54.1-microsoft-standard-WSL2, Rust1.94.0. Exact SSH origin recoweft/quent.
Ancestor/repo and nested repo AGENTS searches returned none; initial tree clean,
one registered worktree, no open repository PRs. External worker/controller leases
are not verified by these bounded checks. Initial branch was the prior PR38 branch
at4c0f0f9; local main was not treated as the remote main. Approved ls-remote/fetch
established actual main3c838b55ede472d466a41d179d64056fcc7d8368. Its log includes
PR22/23/24/37/38 merges; ancestry includes4c0f0f90ec19c7281965d16c2b127e7ea992f92c,
and API independently returned PR38 merged=true with that merge commit. The requested
feature/qt-ai-10-command-ledger branch was created from this verified origin/main.
Git branch creation required approval because .git is read-only in the sandbox;
approved execution succeeded. No user changes were discarded.

Sandbox Git DNS and Docker socket access failed; approved bounded Git remote and
Docker diagnostics succeeded (Docker29.8.0). Worker API repository push permission
and specified-branch SSH push dry-run succeeded independently of Issue credentials.
Actual PR creation/push are reported after their respective operations, not inferred
from a repository permission bit. Sandbox loopback bind failed EPERM; approved
regressions run the actual listener/FIFO controls. No global network/DNS/WSL/Docker
configuration was changed. Docker/remote/branch diagnostics alone are not DB/build/
Windows acceptance. No approval denial was bypassed. No other repository/task,
subagent, merge, Issue closure, release, publication or Issue36 change was performed.

Issue1/11 and settings/policy/dispatch/owned/results/authorization/output/typed/Spec
contracts were read before implementation. Specification question
d85b9889-e586-4efc-98e8-2ed10e3e9a31 received its controller-announced answer via exact
get_answer: same_revision adoption of full-column INSERT, tenant supplementation
only, with default/identity semantics not expanded. ScopeFacts is an application
assertion, not live verification; known unsupported states cannot be overridden.

## Finite production evidence and limits

Both real drivers and the same final dispatcher run in execution/tests/command.rs.
An independent SQLx connection observes committed business/ledger state and actual
server waits; fixture setup and observers are not business submissions. The private
production-point recorder counts business submission attempts separately from
ledger_lookup/reserve/result and begin/begun/commit/committed/rollback/rolled_back.
It is not a packet counter. The trusted provider probe has a real acquisition
positive control (calls1/responses1), followed by zero increments across commands.
No command API can register/invoke or restore authorization.

| Cases | Observation / boundary |
| --- | --- |
| C01/C08/C23 | DB/provider-free preparation; existing Plan/review/Spec/CLI/MCP boundaries retained; public claims cannot restore command authority |
| C02/C06/C10/C19 | missing/broad UPDATE proof, wrong tenant/native/NULL/column/full coverage, unsupported table context, protected/generated/PII columns refuse before ledger reservation/business |
| C11/C17 | both all-column explicit-tenant and supplemented-tenant INSERT first commits, then Low UPDATE first commit; full supported result stored before commit; separate observer reads business and binary input/result |
| C17 replay | each initial operation has new-session execute replay and explicit lookup: business0, ledger lookup1, begin/commit ACK; same complete input is required |
| C17 identity | different values, empty/NULL, zero/false changes, implicit-vs-explicit tenant, write order and operation kind conflict; namespace/tenant/target separation yields explicit lookup miss, business0 |
| C12/C14 authority | exact operation substitution, subject/session/context mismatch and elapsed TTL reject; same-generation changed schema order/policy content or subject in a new session conflicts without disclosing result |
| C13 | unpolled futures consume handles; diagnostic-checked move/clone/copy/private/serde/public-view/byte/result-restoration negatives with actual compiling controls |
| C16 atomic | business constraint/duplicate errors, full affected-result mismatch and real ledger result CHECK failure roll back reservation + business; independent observer sees prior business value and no reservation |
| C17 minimum reservation race | two actual production command transactions, winner paused after real reservation at test-only final await; loser observed in server unique-reservation lock wait; winner business1/commit, loser business0/LedgerFailed/ACK rollback; observer sees one row/ledger; subsequent explicit lookup replays |
| C18 storage | unknown canonical header, corrupt result and NULL pending reservation read from real DB refuse, business0; deliberately invalid fixture ledger version2 also refuses before NULL can become missing; unit recognizer additionally rejects malformed tags/native lengths/bools/UTF-8/depth/size/truncation/trailing changes |
| C13/C14 cancellation | actual PG/MySQL UPDATE wait observed before drop, runtime timeout and disconnect: business1, no claimed commit/ACK rollback, session quarantined; separate test-only pending final await distinguishes business0 timeout/drop |
| C14 expiry | actual pending test-only final await observed after reservation; real monotonic time expires, resume -> Expired, business0, acknowledged rollback; no product skip-clock/force-allow |
| C16/C18 completion | existing test-only begin/commit/rollback/reply-loss seams retain OutcomeUnknown/quarantine; successful native commit with injected lost reply is explicitly reconciled in a new session; not physical lost-reply evidence |
| C15 output | constant redacted carriers, fixed errors/source=None, bounded public state views, actual lifetime tracing capture empty with emitting detector positive control; existing notice/panic/CLI/Windows local-detail refusal controls retained |

Each dialect's main printed core summary: initial commits4 (two INSERT, one UPDATE,
one minimum-race winner), new-session replay/lookup7 with business0 each, and11
counted ACK rollback cases plus additional explicitly asserted conflict/refusal/
completion cases. This summary is a labeled subset, not a total assertion count.
The injected successful-native-commit/lost-reply experiment adds its own committed
business operation and reconciliation outside those counters. Provider totals stay
at their positive-control1/1; command deltas0/0. Lookup/read-only transactions are
counted separately from business writes. Cancel/unknown experiments are not counted
as confirmed rollbacks merely because final observer values remained unchanged.

Fixture catalog observations verify no default/identity/generated/RLS/trigger in
these new dedicated tables and MySQL InnoDB on both tables. Production code does not
perform or infer that catalog verification. Its guarantees remain conditional on
correct complete app schema, physical routing, engine, settings, privileges and lack
of external effects. The disposable setup role owns dedicated fixture DBs; deployment
least privilege/grants/backup/retention are application responsibilities.

PR2 still owns detailed race interleavings, optimistic command version/results and
physical commit-response-loss recovery. PR3 audit/outbox and external delivery are
not implemented. No exactly-once, external-effect rollback, live authentication,
TLS handshake, malicious historical rewrite detection, concurrent DDL safety or
post-retention deduplication is established. Hosted Windows DB-free/compiler/CLI
results are separate from local WSL tests; local Windows process/SDK/DB/ACL execution
is not inferred or claimed.

## Verification commands

Development failures are excluded: the first expanded live run caught a missing test-only command_ready
observation event; the event was added at the actual pre-business boundary. Both
failed containers/volumes were removed and absence checked. The repeated live run
passed both dialects before the final full regression.

| Final local command / selection | Result |
| --- | --- |
| Default workspace test |95 passed,0 failed/ignored/filtered (93 retained +2 Core) |
| All-feature DB-free `--skip live_ --test-threads=1` |213 passed,0 failed/ignored;14 substring exclusions (including new live suites) |
| Exact incidental Core context / manifest claim / CLI detecting control |1 passed each; no zero-selected complement counted |
| Final digest-pinned full workspace, single test thread |227 passed,0 failed/ignored/filtered (222 retained +2 Core +1 decoder/output +2 live suites); cleanup absence verified |
| Shared real compiler harness |294 diagnostic-checked negatives +16 compiling controls (272+14 retained) |
| Fresh generated-consumer harness |76 negatives +8 controls in6 invocations retained |
| `bash -x scripts/check-features.sh --offline` |262 checks and3 standalone CLI test commands passed |
| Existing30 facade +8 CLI combinations |76 sequential test / all-target warning-denied Clippy commands passed |
| Default/all-feature workspace builds |passed locked/offline |
| Final workspace all-target/all-feature Clippy `-D warnings` |passed |
| Final all-feature rustdoc, no deps, `RUSTDOCFLAGS=-D warnings` |passed |
| fmt / diff whitespace / Core/default dependency trees |passed; Core zero deps, default facade/CLI local Core only |

Compiler/CLI build-run commands were sequential. No dependency/feature/manifest/
lockfile/toolchain/edition/publish=false or hosted workflow change. No original
fixture/runtime_pending marker was modified. Initial development compilation also
caught internal macro field visibility, corrected before successful builds. Approval
was requested for the specific remote/.git/Docker/loopback operations and approved
execution succeeded; no denied or pending approval remains in these observations.
PR22/23/24/37 were additionally confirmed merged=true/base main by API, and the
pre-push remote recheck still returned main3c838b55 and no competing open PRs.

Final disposable servers: PG16.14 UTC/read committed/UTF8/en_US.utf8;
MySQL8.4.8 +00:00/REPEATABLE-READ/utf8mb4/utf8mb4_bin with
ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,
ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION and retained SQLx FOUND_ROWS.
Every dedicated run, including the failed development run, removed its containers
and anonymous volumes and verified absence. No existing database/prune was used.
Actual new-head hosted push/PR run IDs, job conclusions and both required DB logs
are separately recorded in the RelayWeft report, not inferred from local success.
