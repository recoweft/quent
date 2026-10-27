# QT-AI-10 planned PR2 finite measurements

Work `cf48404b-dadd-4d84-9ae7-497d473389ac`, revision2, public execution
`e8c2e11c-75c9-4ab6-b925-c4705aae1485`. WSL measurements 2026-10-27 JST.
The exact pushed head, actual PR URL, final hosted CI and report receipt belong to
the fixed RelayWeft report; a PR body is not a report receipt or user acceptance.
[Current contract](command-ledger-v1.md), [historical PR1 evidence](command-ledger-v1-evidence.md).
All24 original full runtime_pending markers and original fixture contents remain.
PR3 audit/outbox is not implemented.

## Identity and authorized environment

Actual cwd `/home/murai/github/quent`, Ubuntu24.04.5 / WSL2 kernel
6.18.54.1-microsoft-standard-WSL2, Rust1.94.0. SSH origin recoweft/quent.
Ancestor/nested AGENTS searches returned none; initial tree clean and one registered
worktree. All current changes belong to this execution. No external worker/lease
introspection or local Windows execution is claimed. Initial and rechecked actual
remote main is `b9ef6dbfcc0f26e3e0f0c1ec1fe1061f9b1c2176`, the requested branch's
base, rather than assuming local main is current. PR39 is API-confirmed MERGED,
head `30a8be058f7c7c4690fb9f00638cb36b62e46b54`, merge the exact remote main.
PR22/23/24/37/38 are API-confirmed MERGED/base main; main ancestry contains their
integration chain. No open repository PRs or second registered local worktree were
observed before implementation/rechecked before submission. This does not establish
external worker lease ownership.

Sandbox DNS/Git and Docker socket diagnostics failed. Narrow approved remote and
Docker diagnostics succeeded (Docker29.8.0); worker login/repository push permission
and specified-branch SSH push dry-run succeeded independently of Issue credentials.
Approved dedicated DB/loopback/FIFO regression executions are separate from those
read diagnostics. No denial was bypassed or global network/DNS/firewall/WSL setting
changed. Git writes/push/PR and actual hosted jobs are reported after execution,
not inferred from a permission bit. No merge/Issue close/release/publish was requested
or performed. Issue36/source-review head coverage are not modified.

Issue1/11 and command encoding/evidence, policy/settings/dispatch/owned/results/
authorization/output contracts and implementations were read first. Revision2 carries
forward rollback race optionA: an independently started waiting attempt may acquire
its own reservation after the leader rolls back, even with different input.
Question `02924154-ff13-425c-acf3-613a6349de25` was answered through the controller's
announced exact-ID get_answer: same-revision exact1 only. Unsupported expectations
0/2..256 precede any ledger read, not IdentityConflict. Draft exact0/QCC2/QCR2 work
was removed and its experimental success is excluded from final acceptance.

## Production, real DB and real communication measurements

The recorder is at the existing production business dispatcher, retaining SQL
allocation address/full ordered native args/metadata. Ledger/provider/Tx events are
separate; setup/observer queries are not business sends or driver packet counts.
Both real SQLx drivers use the same private finalized preparation, binder and
owned transaction. There is no alternate business renderer or unsafe executor.
Independent connections observe committed business, binary ledger input/result and
server waits for the exact connection ID. All cases use new disposable namespaces/
roles/settings and digest-pinned servers; no existing DB is consumed.

| Cases / QT acceptance | Each engine's measured finite result |
| --- | --- |
| C01/C02/C06/C08/C10/C11/C19/C23 regressions | Strict/policy/tenant/key/protected table/native/subject refusal and existing Plan/Spec/review/CLI/MCP DB/provider0 boundaries retained |
| PR1 compatibility / C17 | frozen canonical keys/inputs from unmodified merged PR1 Core; exact byte equality with both implicit and explicit exact1 APIs, real old-record seeding, saved QCR1 replay/business0 twice; not a historical deployed DB claim |
| version100/101/102 / C16/C17 | three business attempts: version0 succeeds to1, stale version0 Conflict/ACK rollback, version1 succeeds to2; two successful saved-result replays/business0; exact retained SQL address/full args/metadata and binary saved result checked |
| native width controls / C16 | MySQL I8/I16/I32/I64, PG I16/I32/I64: explicit full-column INSERT, version native max-1 to max, saved replay/business0 and independent exact native max observation; checked overflow constructor refusal before DB; PG I8 preparation refusal explicitly separate |
| wrong asserted physical key / C16 | deliberately non-unique real fixture with asserted Single facts: ordinary and version UPDATE each matched2 -> ExpectationMismatch/ACK rollback, observer values unchanged, no reservation/result; does not support inaccurate physical schemas |
| exact expectation / C16 | expected0/2/256 × INSERT/ordinary/version (nine boundary samples per DB): validation Unsupported and Tx/ledger/business/provider0, including existing durable ID; no different-supported-expectation IdentityConflict positive exists |
| missing target / C16 | ordinary0 -> ExpectationMismatch, version0 -> Conflict; one business send each, ACK rollback, result/commit events absent, independent observer reservation0 |
| version/current substitution / C12/C17 | changing version after validation -> OperationMismatch/NotStarted/business0; same retained ID with a different supported version -> IdentityConflict/ACK rollback/business0 |
| matched vs changed / C16 | unchanged PG native control1/production matched1; unchanged MySQL native handshake control with FOUND_ROWS cleared0/production FOUND_ROWS1; actual observer value unchanged |
| races110/111 / C17 | leader reserved, follower actual unique wait observed by server, leader business1/commit ACK, follower business0/LedgerFailed/rollback ACK; fresh session/current validation same-input replay or different-input IdentityConflict, business0 |
| races112/113 / C14/C17 | server-observed unique wait, leader real-clock expiry/ACK rollback/business0; independent follower same OR different input reservation succeeds and FirstCommits/business1; one retained ledger; vanished input is not remembered |
| races114/115 / C14/C17 | leader actual UPDATE row wait plus follower actual unique wait both server-observed; real-clock expiry after leader business send, unlock -> ACK rollback; independent same/different follower business1/commit ACK; **two total business sends**, one retained ledger/result, independently observed final value |
| unresolved leader / C17/C18 | four explicit new-session lookups while leader reservation uncommitted return None/business0; no write retry inferred; subsequent outcomes are separately controlled/observed |
| network120 / C18 | actual COMMIT frame stopped before forwarding, no server ACK, client OutcomeUnknown/quarantine; fresh lookup None, observer business unchanged and reservation0; not a rollback ACK claim |
| network121 / C18 | actual COMMIT frame forwarded, real PG CommandComplete COMMIT / MySQL OK consumed and never sent to client; production transport error -> OutcomeUnknown/quarantine; observer committed business + exact input/QCR1/version1, fresh-session/current lookup Replayed/business0 |
| network122 / C18 | COMMIT forwarded and server reply forwarded; production FirstCommit, observer committed business/input/result/version, fresh lookup Replayed/business0 |
| storage22 / C18 | actual corrupt binary storage: unknown header, truncation, invalid tag/native width/bool/UTF8, oversized input, extra/duplicate semantic content, unknown/truncated/oversized/trailing/wrong-dialect/count0/count2 result, NULL input, pending, unknown/NULL ledger version; every case business0/reserve0/ACK rollback, restored valid record Replayed/business0 |
| storage native/duplicate / C18 | two same-key records -> LIMIT2 refusal; native INTEGER1 instead of SMALLINT1 -> LedgerInvalid, despite MySQL raw i16 decode succeeding as a detecting control; both business0/ACK rollback |
| C13/C14/C15 | prior unpolled/drop/timeout/server-wait disconnect/expiry/borrowed refusal and completion seams retained; private expectation field negative + compiling explicit exact1 control; actual lifetime tracing empty with emitting canary detector positive control, fixed errors/sourceNone/redaction/public views retained |

The trusted provider probe has a real acquisition control (calls1/responses1), then
no increments over command cases; Low commands add no provider entry/authorization.
Exact-one is sealed in the private handle and fixed QCC1 protocol; version original/
final operation and ordered successor/expected args are retained in the same record.
Result stays QCR1/13 bytes/dialect/count1. Expectations add no stored payload budget;
version uses existing IR/native-argument budgets (successor and expected arguments).
No RETURNING/native-family/High command/batch/upsert expansion or installation DDL
change. Explicit binary payload metadata and exact signed SMALLINT ledger version
are checked before decoding; SQLx byte/integer compatibility is not native proof.

The dedicated `cfg(test)` proxy only frames one plaintext loopback fixture
connection, bounds frames at1MiB and read/write timeouts, and never prints driver
payloads. Native COMMIT forwarding/real server response flags are separate from the
business recorder. It never changes production code to force an error or modifies
global networking. The earlier successful-native-commit-then-forced-error seam is
retained as **test seam**, not physical lost-reply evidence. Explicit reconciliation
is a new validation on a new production connection, never a failed Tx/snapshot read
or automatic business retry/new ID. Existing session quarantine/ACK-only completion
and consumed-future/drop ownership boundaries remain.

## Final verification

Final digest-pinned full workspace invocation: **227 passed, 0 failed/ignored/
filtered**, including the11 live DB tests,3 incidental name matches, and compiler/consumer/CLI/
output/manifest/typed/Strict/source/review/regen regressions. Dedicated containers
and anonymous volumes removed; exact owned resource absence verified by the harness.
The existing compiler harness now asserts **295 diagnostic-checked negatives +16
compiling controls** (294+16 retained); generated consumer retains **76 negatives +8
controls in6 invocations**. No compiler/CLI build-run was parallelized.

| Additional completed check | Result |
| --- | --- |
| Default workspace |95 passed,0 failed/ignored/filtered |
| DB-free all-feature workspace, `--skip live_ --test-threads=1` |213 passed,0 failed/ignored,14 excluded; not DB acceptance |
| Exact incidental Core context / manifest nonpromotion / CLI detecting control |1 selected passed each; other name-filtered tests not counted as selected |
| Shared compiler |295 diagnostic-checked negatives +16 compiling controls, including private expectation E0616 and explicit exact1 control |
| Generated consumer |76 negatives +8 controls /6 invocations retained, no dependency/import error counted as a negative |
| `bash -x scripts/check-features.sh --offline` |262 checks +3 standalone CLI test commands passed |
| Existing30 facade +8 CLI feature configurations |76 sequential test / all-target warning-denied Clippy commands passed |
| Default/all-feature builds, fmt, diff whitespace |passed locked/offline |
| Workspace all-target/all-feature Clippy, all-feature rustdoc |passed, warnings denied |
| Normal dependency trees |Core zero dependencies; default facade/CLI only local Core |

Hosted CI on the actual pushed head is separate evidence, especially Windows
DB-free/compiler/CLI/output versus WSL's real DB/network tests; the final report
records observed jobs and head. Original Cargo.lock/manifests/toolchain/edition2024/
publish=false and workflows are unchanged.

Development failures are excluded from the successful totals: first new PG i8
assertion expected the wrong fixed wrapper classification (actual Policy InvalidPlan
Unsupported); next native-width fixture reused a cached observer query after DDL
changed its result type. The assertion and fixture were corrected, and a fresh
production connection is used for the intentionally wrong-native ledger installation.
Failed runs still removed their dedicated containers/volumes. The final full run
above passed after these corrections; no skip/0-selected check replaced acceptance.

## Limits retained

These are finite production/DB/transport experiments, not controller independent
retests or exactly-once guarantees. Arbitrary external networks, TLS handshakes,
physical fault generality, every cancellation packet boundary, local Windows DB/
filesystem integration, live production least privilege, external authority stores,
routing/DDL races, adapter state and malicious history/retention deletion are not
verified automatically. Apps must durably retain ID/input/context/ledger before
first submission and supply authentic current authority on reconciliation. Changed
subject/schema/policy must not disclose old results. No rollback-deleted historical
input conflict memory, retained-history-free deduplication, external callback/
delivery atomicity, PR3 audit/outbox or Issue-wide completion is claimed.
