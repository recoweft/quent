# QT-MAINT-01 PR1: finite authorized cancellation controls

Work `28771c83-fe58-44a9-9552-87f76a05e899`, revision 1, public execution
`555de916-4fea-4522-88a0-00f831a62a4c`; measured 2026-10-27 JST.
Exact pushed head, actual PR and hosted CI are recorded in this execution's
RelayWeft report. Receipt is not acceptance. Refs #44, #43.

## Original failure remains evidence

The controller's supplied PR43 evidence records source head
`fc4d9dbc299c70c780be22a66f0a0836123bec3a`, push run40459329048 (82 successes),
and pull_request run40459334483 attempt1 (81 successes, one failure). Required DB
job122039962830 stopped after 39 passes/one failure, with subsequent suites
unexecuted and cleanup successful. `live_authorized_postgres` observed updated
instead of prior state. The preceding completed log was direct mode0; direct
timeout mode1 is the controller's inference, not a newly recovered detailed mode.
The authorized fixture blob `88c4e3849e9a08ea10991af97bd3b7903d67e5e9` is identical
in PR43 and its main base. This is an existing cancellation-order race, not a
migration planner behavior change.

Retry job122042778771 and push DB job122039945239 each had 252 passes and no
failures/ignored/filtered, with PG16.14/MySQL8.4.8 and successful cleanup. Native
hosted Windows job122042808293 had 112 default +226 DB-free passes/17 filters.
These are historical controller observations, not new-head results. The supplied
all-attempt audit distinguishes 164 currently effective successes (82 push +82
PR), one historical failure, and 165 actual executions: 81 retry jobs carried
forward their original timestamps; only Required DB executed again. A green
rerun does not remove the original failure or demonstrate fixture stability.

## Environment and scope

Actual cwd `/home/murai/github/quent`, Ubuntu24.04.5 on WSL2
6.18.54.1-microsoft-standard-WSL2, rustc/cargo1.94.0. No applicable ancestor or
repository AGENTS.md found. Initial tree clean. SSH origin is
`git@github.com:recoweft/quent.git`. Local initial HEAD was the old PR43 source;
remote main was fetched independently and verified as
`cdc2ddfe191e8446b2481abaadf52ff17cec2628`, containing that source. GitHub PR43
was independently verified MERGED with that merge commit. Issue1 and Issue44
full body/comments were read through worker GitHub authentication; Issue44 had
no comments. Specified branch was created from fetched remote main.

Sandbox DNS/GitHub and Docker socket checks failed. Narrow approved diagnostics
succeeded: remote main, worker identity recoweft, repo push/admin permission,
SSH specified-branch push dry-run, Docker29.8.0 and both cached image digests.
The sandbox gh status message alone did not establish invalid credentials.
No approval was denied. Approved actual pushes/PR/CI are recorded separately;
permissions/dry-run alone do not prove a PR was created. No global network,
firewall, TLS, WSL or Codex setting was changed.

Changes are test-only fixtures and documentation. Production API, dispatch,
linear carrier consumption at future creation, fixed errors, quarantine,
no automatic retry, atomic completion semantics, drivers/pins/features/lock,
ordinary CRUD/command and migration boundaries remain unchanged. No migration
PR2/preflight/apply, Issue36 Git-head work or external delivery is included.
All 24 full runtime_pending markers remain.

## Controls and independent observations

Each case below runs on both production SQLx drivers, the common authorization
execution gate and an independent observer. Provider invocation, business-send
recorder and Tx completion events are separate quantities. Existing full ordered
args/SQL-allocation/all-metadata comparisons, native/NULL, public canaries,
provider refusals, expiry, ownership and ACK completion controls remain.

| Case ID | Finite order / measured result per DB invocation |
| --- | --- |
| MAINT-CANCEL direct mode0/1/2 | locker holds row → real server lock wait → drop/actual timeout/disconnect → explicit fixture termination → independent backend absence → locker ACKrollback → observer prior state; provider1/business1, available=false, no commit/rollback ACK of canceled session |
| MAINT-CANCEL owned mode0/1/2 | same order and prior-state observation; begin1/begun1/business1; no commit or acknowledged rollback; disconnect returns OutcomeUnknown/atomic OutcomeUnknown; drop/timeout return no confirmed disposition |
| MAINT-UPDATE hold=false | actual prepared ordinary UPDATE forwarded once; actual server completion forwarded; independent observer updated; provider1/business1, Tx0, SQLx Observed/decoded, session available; withheld=false |
| MAINT-UPDATE hold=true | actual prepared ordinary UPDATE forwarded once; actual server completion consumed and withheld; independent observer updated while future remains pending, then actual timeout/drop; provider1/business1, Tx0, decoded0, unavailable, no commit/rollback events; withheld=true |
| Retained pre-send controls | unpolled dispatch business0; provider timeout provider1/response0/business0; final expiry/refusals business0; no consumed handle returned |
| Retained ACK controls | owned success begin/begun/send/decoded/expectation/commit/committed; changed-value expectation mismatch explicit ACKrollback and independent prior-value observer; borrowed result remains Uncommitted |

For mode0/1 the fixture terminates the connection *after local cancellation while
retaining the quarantined session*, then drops it. For mode2 the existing server
termination triggers the driver's unknown error. PostgreSQL pid/current_user and
MySQL PROCESSLIST_ID identify only the fixture connection. Backend absence is
queried separately, with a 10-second deadline, while the locker still holds the
row. Missing lock wait, termination failure, absence timeout or wrong observer
state fails; no sleep extension/retry/ignore/assert relaxation is used. Locker
rollback ACK is never confused with canceled-session rollback ACK. The final
prior-state read is a finite fixture result, not a production rollback guarantee.

`authorized_network.rs` is nested in cfg(test) execution tests. It uses one
plaintext loopback connection, bounded accept/connect/read/write and a capped
protocol frame reader shared with the existing command fixture. Handshake/prepare
responses flow normally. PostgreSQL extended Execute and MySQL
COM_STMT_EXECUTE are counted once; PostgreSQL UPDATE CommandComplete (one affected
row) / MySQL OK (one affected row) are recognized. The hold control consumes that
response and subsequent ReadyForQuery without delivering it to SQLx; continued
server reads allow autocommit to finish. Independent committed visibility, rather
than the protocol response alone, proves application. The pass control detects
forwarding/decoded success and the hold control detects the opposite completion
boundary. Both demand the updated value; the termination controls demand the prior
value. Thus swapping/removing either control's order cannot pass by allowing both
values. No post-execution arbitrary error seam substitutes for network evidence.
This is ordinary UPDATE completion, distinct from native COMMIT proxy tests.
No payload is logged, retained as an artifact or exported via public output.
TLS is disabled only on these dedicated fixture connection options.

## Fixed repetitions and regression record

Before running either target suite, the plan was fixed to three numbered
invocations of `scripts/test-dispatch.sh cargo test -p quent --all-features --lib
live_authorized_ --locked --offline -- --test-threads=1 --nocapture`, with every
exit/log retained and no failed trial replaceable by a later success. Each
invocation creates fresh dedicated digest-pinned DB containers via the unchanged
script. The three planned trials all executed; no extra target trial was needed.

| Trial | PostgreSQL | MySQL | Other tests | Cleanup |
| --- | --- | --- | --- | --- |
| 1 | suite passed; 6 termination controls prior, pass+hold controls updated | same | 38 filtered, not counted as passed | containers/anonymous volumes removed, absence verified |
| 2 | suite passed; same 8 observations | same | 38 filtered, not counted as passed | removed/absence verified |
| 3 | suite passed; same 8 observations | same | 38 filtered, not counted as passed | removed/absence verified |

Each trial: 2 passed,0 failed,0 ignored; per DB three suites and 24 listed control
observations total (18 termination +6 response controls). Existing top-level test
count is unchanged: two response cases per DB were added *inside* the existing
suite; the six cancellation cases per DB were strengthened. Full regression
therefore remains the 252 baseline, with four additional response-control scenario
executions across both DBs per full run, not four additional Rust tests.

Target logs were retained for this execution (byte lengths / SHA-256):

| Trial | Bytes | SHA-256 |
| --- | --- | --- |
| 1 | 4429 | `fee5295c237e51bfe5316b0b076d6bbfae381403380f0bdfc81052f0c7a258f2` |
| 2 | 4429 | `4368a17b6c1c5e45b95cc2dd00313d7b3dd6bfaf7c50f3316d7ae7ca98e7c238` |
| 3 | 4429 | `1269b9789c12620c637037b23124c08c4344b36be6b38343554d92c8754c2a55` |

The unchanged script uses PG digest
`sha256:16bc17c64a573ef34162af9298258d1aec548232985b33ed7b1eac33ba35c229`
and MySQL digest
`sha256:2952e3be7807f06fc18de50b3ea1a632d5c70d63482ff7d7376fe3aa8999babf`.
All trials observed PG16.14, UTC, read committed, UTF8/en_US.utf8; MySQL8.4.8,
+00:00, REPEATABLE-READ, utf8mb4/utf8mb4_bin, ONLY_FULL_GROUP_BY,
STRICT_TRANS_TABLES, NO_ZERO_IN_DATE, NO_ZERO_DATE, ERROR_FOR_DIVISION_BY_ZERO,
NO_ENGINE_SUBSTITUTION. Existing FOUND_ROWS control and default InnoDB table
semantics are retained. Limited roles, dedicated namespaces and alternate/hidden
catalog fixtures are unchanged; no global privileges/settings/prune were added.

Final local commands (compiler/build-run commands sequential, locked/offline):

| Command / selection | Result |
| --- | --- |
| `scripts/test-dispatch.sh cargo test --workspace --all-features --locked --offline -- --test-threads=1 --nocapture` | 252 passed,0 failed/ignored/filtered; all 12 termination and four response scenarios observed; cleanup absence verified |
| `cargo test --workspace --locked --offline -- --test-threads=1` | 113 passed,0 failed/ignored/filtered |
| shared `owned_handle_compile_contracts` inside full run | 347 diagnostic-checked negatives +19 compiling controls, count assertions passed |
| fresh generated-consumer harness inside full run | 76 diagnostic-checked negatives +8 controls, six invocations |
| fmt; default/all-feature workspace build | passed |
| all-target/all-feature workspace Clippy `-D warnings`; rustdoc `RUSTDOCFLAGS=-Dwarnings` | passed |
| `bash -x scripts/check-features.sh --offline` | 262 cargo checks +three CLI test runs passed |
| existing CI30 facade +8 CLI combinations, each test then all-target `clippy -D warnings` | all 76 sequential commands passed |
| Core/default facade/default CLI normal dependency trees | Core empty; facade and CLI only local Core |
| diff whitespace; pins/lock/workflow/script/full fixture comparison to main | passed; original fixture bytes identical,24 pending markers |

Four local DB invocations total (three fixed target +one full) each verified both
container and owned anonymous-volume absence. No failed trial, skipped test or
zero-selected invocation is included as successful evidence. No implementation
compile/test failure occurred in these local runs; expected compiler-negative
errors are checked assertions, not failed test cases. No approval was denied;
sandbox failures and successful approved execution are distinguished above.
Native hosted CI belongs to the actual pushed-head report, not this local table.

## Limits

Uncontrolled post-send autocommit cancellation remains unknown to the client:
server termination/lock-release order can permit an applied write despite no
completion ACK. Quarantine, missing commit event, socket drop or missing rollback
ACK are not rollback evidence. An owned explicit Tx with no completion ACK is
not promoted to RolledBack either. Controlled test termination cannot provide a
new production cancellation protocol or exactly-once guarantee. No ordinary
operation is automatically resent. No uncontrolled observer result is accepted
by an unconditional prior-or-updated assertion in this change.

These finite plaintext SQLx/server observations do not establish arbitrary
protocol/TLS/proxy interoperability, external provider authentication/cancellation,
physical routing, concurrent DDL, local Windows SDK/GUI/export ACL or Windows live
DB behavior. Native hosted Windows checks must be identified separately from WSL
DB results. Issue44 acceptance and integration remain reviewer/controller decisions;
worker does not merge, close, release, publish or start the next task.
