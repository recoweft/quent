# QT-AI-10 planned PR3 finite evidence

Work `0e512108-5def-4ea7-a734-0f64907df161`, revision1, public execution
`b6674838-196e-4836-9f3c-aa7d31d4bd55`. WSL measurements 2026-10-27 JST.
[Contract](command-bundle-v1.md), [plain/PR2 contract](command-ledger-v1.md).
The RelayWeft report records actual PR URL/exact head/hosted jobs/report receipt;
a PR body is not a report receipt or acceptance. All24 full runtime_pending markers
and all original fixtures, including the four frozen PR1 bins, are unchanged.
No Issue closure, merge, release or publish is performed.

## Identity, prerequisites and approvals

Actual cwd /home/murai/github/quent, Ubuntu24.04.5 / WSL2 kernel
6.18.54.1-microsoft-standard-WSL2, Rust1.94.0. SSH origin recoweft/quent matches the
fixed request; initial tree clean, one registered worktree. Repo/ancestor/nested
AGENTS searches returned none. These changes belong to this execution. Bounded
process-cwd inspection saw only the current sandbox worker/tools; it does not
establish ownership/absence of external controller leases or workers.

Approved remote ls-remote/fetch established actual main
`792c288c3892c375a8085ff0fd761fb8cf2756e5`, rather than assuming stale local main.
API independently confirmed PR40 merged=true/base main/head
`decbc1ef9009573ee0445f0dc19b1bced355d354`/that merge commit; Git verified ancestry.
The main log contains PR22/23/24/37/38/39 integrations; API separately confirms all
seven prerequisite PRs merged/base main. Pre-push remote main remains the same SHA,
the requested remote branch is absent and Issue11 is OPEN. No open PRs or second registered worktree were observed at start. The requested
branch was created from this verified origin/main. Issue36 and unrelated source
review head-coverage work are unchanged.

Sandbox DNS/Git and Docker socket checks failed. Narrow approved read diagnostics
succeeded: actual worker API account/repository push permission, specified-branch
SSH push dry-run and Docker29.8.0 response. The sandbox failures were not treated
as invalid host credentials; no network/DNS/firewall/WSL/global Codex settings were
changed. Actual remote mutations and approved dedicated test runs are separately
recorded in the final report. No approval denial was bypassed. Local Windows launch,
SDK/build/GUI/export checks were not attempted or inferred from Docker/WSL success.
Hosted Windows DB-free/compiler/CLI/output results are separate from local DB evidence.

Issue1, supplied whole Issue11, both predecessor evidence documents and Core
canonical/IR/policy/settings/dispatch/owned/atomic/authorization/output/typed contracts
were read. Controller-delivered answer to question
`34fb1571-7211-40e9-8d03-409f107a0506` specifies same_revision option A: the original
valid business context is the sole authority, with explicit finite companion Tenant
mapping held outside normal PolicySet. Existing constructor and ordinary/plain
Column equality are preserved. No independently authenticated companion contexts or
general context rebinding API were added.

## Production correspondence and per-engine measurements

The new `execution/tests/command/bundle.rs` runs the actual production SQLx drivers,
owned Tx and common retained-record dispatcher. Inspection SQL allocation addresses,
full native args and Metadata match all three actual sends for full-column INSERT
and optimistic UPDATE bundles. Original context/mapping and each private final
record are retained together. No alternate renderer/binder or caller executor exists.
Test-only observation distinguishes record submissions, responses, ledger stages and
Tx ACKs; setup/observer SQL and protocol frames are separate from business sends.
Provider acquisition has a real calls1/responses1 control; all subsequent bundle
cases leave those counts unchanged (command increments0/0).

Each engine uses a new dedicated catalog/schema and role in digest-pinned PG16.14
or MySQL8.4.8. All four tables are catalog-observed for absence of generated/default/
identity columns and user triggers (PG RLS also absent); MySQL explicitly observes
four InnoDB tables. These fixture observations do not authenticate production
DDL/routing/schema assertions or role least privilege.

| Case / QT case | Measured per-engine result |
| --- | --- |
| B01/B02 / C16/C17 | explicit/omitted equal tenant in opposite companion positions; explicit native NULL and actual canary payload; business1/audit1/outbox1, ledger lookup1/reserve1/result1; three native write responses and three ledger responses; begin/begun/commit/committed; all four committed tables and strict binary input/QCR2/version2 independently observed |
| B01/B02 replay / C17/C18 | fresh production session execute then explicit lookup: Replayed, business0/audit0/outbox0, lookup-only ledger responses, no reservation/result save; counts/records unchanged |
| input conflicts / C17 | audit/outbox payload, record/event ID, sequence, write-column order, omitted vs explicit tenant, role/table placement; same key, IdentityConflict/ACK rollback, all record sends0; no overwrite or missing fallback |
| plain ↔ bundle / C17/C18 | same main key/namespace/ledger, both directions IdentityConflict; a plain QCC1/QCR1/version1 commit remains readable by the new protocol and receives no retrospective companion rows |
| current/mapping / C11/C12/C14 | post-validation input substitution, same-generation schema/subject/target change, TTL expiry, unpolled future drop: NotStarted, begin/ledger/all records0; new-session changed subject with corresponding valid link input conflicts without disclosure |
| eligibility / C02/C06/C08/C10/C19 | tenant mismatch/NULL/wrong native/duplicate, nullable/wrong-table/wrong-width/missing mapping, duplicate roles, missing schema/table semantics, protected/generated/PII/required-filter, omitted non-tenant column, per-payload oversize, reused ledger/business/audit/outbox table: pre-DB refusal, Tx/ledger/all records0 |
| aggregate budget / C24 | two companion operations each with80 additional explicit4096-byte Text values; each finite operation prepares, aggregate canonical refuses at1MiB before begin; no truncation/send |
| V70/V71 / C16/C17 | native optimistic business UPDATE version0→1 commits with audit1/outbox1; saved original version0 input replays with all record0 against physical version1; new-ID stale version0 gives Conflict/ACK rollback with business1/audit0/outbox0 |
| F1/F2/F3 / C16 | duplicate business/audit/outbox respectively: submission counts1/0/0,1/1/0,1/1/1; each ACK rollback; reservation absent and original business preserved/no partial newly committed companions |
| F4 / C16/C18 | real result CHECK failure after business1/audit1/outbox1; result-save attempt1, no commit; ACK rollback, independent business/audit/outbox/ledger all0 for this command |
| result counts / C16/C18 | all three stored QCR2 count positions separately changed to0/2: LedgerInvalid/ACK rollback/all record0; restored actual result replays; strict native-result helper rejects unexpected0/2 with ExpectationMismatch before save |
| owned completion seams / C18 | test-only begin error: all record0; pre-native-commit error: all record3 and no committed rows; native-commit-success then forced error: all record3 and all rows committed. All report OutcomeUnknown/quarantine, new-session lookup separately distinguishes presence. These are seams, not physical response-loss evidence |
| actual record waits30..38 / C13/C14 | server-observed business/audit/outbox INSERT waits before drop/timeout/real-clock expiry; sends respectively1/0/0,1/1/0,1/1/1. Drop/timeout quarantine with no ACK claim; release after expiry returns Expired/ACK rollback. New-session lookup/all-table observer show no committed command, no resubmission |
| result-await seams100..105 / C13/C14 | observed cfg(test) pending await before result save or after save/before commit, then drop/timeout/expiry: record sends3, result save0/1 respectively, commit0. Only resumed expiry has rollback ACK; consumed cancellation remains quarantined. Independent records0 and explicit lookup None; not a physical network-fault claim |
| reservation races80/81 / C17 | real server unique wait, winner commits three records, follower LedgerFailed/ACK rollback/record0. Fresh same input replay or different input IdentityConflict; all four retained tables1 |
| reservation races82/83 / C14/C17 | leader observed at existing final pre-business await, follower real unique wait, real-clock leader expiry/ACK rollback/record0. Already independently started same OR different follower proceeds to three sends/commit; fresh replay0 and all four tables1 |
| after-audit-wait races84/85 / C14/C17 | real leader audit INSERT wait and follower reservation wait; leader expiry then lock release leads business1/audit1/outbox0/ACK rollback; same or different follower business1/audit1/outbox1/commit ACK. Two total business and audit attempts, one retained committed ledger/full result and one row in each business/audit/outbox table; fresh lookup0. No permanent memory of rollback-deleted input |
| communication120 / C18 | actual COMMIT frame stopped before forwarding; production OutcomeUnknown/quarantine after three record submissions; observer all four absent, fresh lookup None/all record0; no rollback ACK claim |
| communication121 / C18 | actual COMMIT forwarded and real native server ACK consumed but not sent to client; production transport error OutcomeUnknown/quarantine; independent business/ledger/audit/outbox/result committed; new-session same-ID strict lookup replay sends all records0 |
| communication122 / C18 | COMMIT and actual server ACK forwarded: FirstCommit, observer all four/result present, explicit fresh-session replay all record0 |
| borrowed / C16 | validation Unsupported, owned handle consumed by borrowed execute Unsupported/NotStarted; borrowed recorder empty and caller retains completion ownership |
| output/ownership / C13/C15 | real canary payload/IDs/subject in production sends and fixed Debug/public JSON; outer tracing empty, emitting detector positive control detects canary; existing nested-driver/sourceNone/notices/panic/Windows/CI/MCP controls regress; actual compiler diagnostic negatives and compiling controls verify private/non-Clone/Copy/serde/restoration/move/unpolled-drop boundaries |

The plaintext loopback proxy is the unchanged PR2 cfg(test) fixture on one dedicated
connection, bounded frames/timeouts and no payload logging. It consumes actual PG
CommandComplete COMMIT / MySQL OK responses. Native-success-then-forced-error seams
remain distinct. No TLS/general-network/external exactly-once or global network
configuration modification is claimed. Replay uses a newly validated original input
and supplied current authority, never a failed Tx snapshot or an old version rewrite.
None while another transaction is pending is not rollback proof or retry permission.

## Regression and verification

Final full dedicated-DB workspace:230 passed,0 failed/ignored/filtered. Original
227 retained plus pure bundle test and two new live suites. Includes old frozen four
bins/QCK1/QCC1/QCR1 replay, all PR2 native version widths/FOUND_ROWS changed-vs-matched,
six race/actual communication/storage24 counterexamples; all existing Strict/CRUD/
typed/generated/Spec/authorization/owned/borrowed/catalog/CLI/FIFO/review/source/CI/
regeneration/output controls. Dedicated containers AND anonymous volumes removed,
exact owned resource absence verified, including all development failures.

Default workspace95 passed. DB-free all-feature214 passed with16 explicit exclusions;
three incidental live_-named DB-free controls are run separately and are not DB
acceptance. Shared compiler now319 diagnostic-checked negatives +18 compiling controls
(previous295+16 retained;24 negatives and2 controls added); generated consumer retains
76 negatives +8 controls in6 invocations. Compiler and CLI builds/runs are sequential.
No fail/skip/0-selected test is counted as successful acceptance.

Feature/default/all-feature build, fmt, all-target warning-denied Clippy, warning-denied
rustdoc,262 feature checks +3 CLI test commands and30 facade +8 CLI configurations /
76 sequential test/Clippy commands passed; commands and actual hosted outcomes
are recorded in the final report. Core normal dependency tree remains empty; default facade/CLI only local
Core. Lockfile/manifests/pins/Rust1.94/edition2024/publish=false/feature defaults and
CI workflow are unchanged. Hosted jobs on the actual pushed head, especially Windows
DB-free/compiler/CLI/output and both event live-databases jobs, are separately recorded.

Development failures are excluded from passing totals: an initial observer expected
an already-existing business row to disappear on duplicate rollback (corrected to
preserve it); large test future stack overflow required splitting fixture async
sections; inline companion carrier growth also overflowed an existing plain fixture,
so additional retained bundle records now use a private Box; warning-denied Clippy
required the native is_multiple_of method in a test. Each failed DB invocation still
removed/verified dedicated resources. Final successful tests use the corrected code.

## Limits

This is finite production/DB/protocol evidence, not controller independent retesting
or Issue-wide completion. General physical fault/TLS/arbitrary network timing,
every packet/cancellation boundary (especially begin/lookup/rollback wire loss),
Windows local real DB/ACL/GUI/export, deployment least privilege, authority/current
stores, routing/server replacement, adapter interior state, concurrent external DDL,
malicious privileged history rewrites and history deletion are not automatically
verified. Exact unexpected0/2 companion results use strict result/decoder controls;
ordinary no-effects single VALUES INSERTs natively return1, and no trigger/rule
semantics are promoted to supported just to force another count. External delivery,
callbacks, polling/deliveryack and deduplication after retained history deletion remain
unsupported. App durable input retention/authentication/complete physical assertions
and current snapshots are necessary conditions. All24 runtime_pending markers remain.
