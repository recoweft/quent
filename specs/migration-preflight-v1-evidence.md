# QT-AI-11 PR2 finite evidence

Work `4cccdbc9-34e3-4795-a597-e64fc58c4834`, revision 1, public execution
`ebb82b63-3b79-41db-8589-bf92be6714f4`. This register covers the finite
[phase/preflight contract](migration-preflight-v1.md); it does not accept Issue12,
PR3 application or any full runtime_pending marker. Exact final head/PR/CI results
are recorded in this execution's RelayWeft report and PR, not inferred from PR1.

## Identity and environment

Actual cwd `/home/murai/github/quent`; Ubuntu 24.04.5 on WSL2; Rust 1.94.0.
No applicable repository/ancestor AGENTS.md was found. Started clean on prior
maintenance source `6ce6743e2ab9730c7268b303c0308a8827fdcd52`. SSH origin fetch/push
is `git@github.com:recoweft/quent.git`. Actual remote main was independently read
as `b227132fe46758826f79f19ed317892db1e8c96e`, fetched, and used to create exactly
`feature/qt-ai-11-migration-preflight`. PR1 source
`fc4d9dbc299c70c780be22a66f0a0836123bec3a`, maintenance source, and dependency merges
`4d3c7e3832543af8e9c72f30814c5316e938db71` and
`26377bdada75d5a57a6a87ad35cf9d4ec28bbdba` were checked as fetched main ancestors.
Issue1/12 were read through worker GitHub authentication. Worker identity recoweft,
repository push permission, SSH branch push dry-run and authenticated API/GraphQL
reads succeeded independently from Issue connector credentials. An actual push
and PR creation are separate checks recorded after execution.

Sandbox DNS resolution, Docker socket access and `.git` writes initially failed.
Narrow approved read-only diagnostics succeeded for GitHub main/auth/permissions
and Docker 29.8.0; approved fetch/branch creation and dedicated DB runs succeeded.
No global settings/network/WSL/DNS/firewall or dependency pins were changed. These
checks establish neither Windows process/SDK availability nor live Windows/ACL/TLS.
The delivered consultation answer to `8bcc7b2e-2708-4b2f-b9c9-782c7683f802` was
read with that exact ID and its fixed read-only Tx boundary applied in this same
revision following the controller's continuation instruction.

## Cases and finite measurements

| Cases | Implementation/check | Finite result |
| --- | --- | --- |
| C01/C22 | private phase records and pure request creation | fixed Expand/Backfill/Contract ordering, complete step ranges, no-work distinct from refused/unsupported; repeated exact plan equality; no SQL subset expansion |
| C08/C19/C22 | finite private expected projection and live result | old manifest/public JSON never supplies evidence; unknown default/metadata does not become absent or matched; full effects remain Unverified and overall coverage Partial |
| C12/C14/C22 | independent current context/request/reader and clock comparison | target, plan/after-name, context generation, phase, condition/exclusion generation, role/visibility generation, reader identity mismatch; expired and anomalous time refuse |
| C13/C15/C23 | real compiler harness with compiling API control | private phase/evidence fields inaccessible; Clone/serde/view/old verify summary cannot restore evidence; evidence has no apply method and is rejected by CRUD validation |
| C15 | exact JSON allowlist, detecting canaries and real driver capture | source role/name canary exists; Debug/Display/public JSON omit it; DDL-exclusion generation Debug redacted; outer capture sees no production driver events |
| C19/C22/C24 | both real DB adapters and private malformed-result checks | actual matching SELECT reads; changed live default detected on a fresh read; current recheck alone does not monitor DDL; missing grants, nonunique index, normalized NULL, unsupported metadata type, row/query/string budgets, timeout/disconnect/cancel do not mint success evidence |
| C24 | finite extension records | zero/multiple/duplicate/reordered/wrong namespace/table/role/field shape/count tokens fail; 1- and 2-table observations exercise explicit scope and shared meter |

Pure DB/provider zero is a structural guarantee: Core has no dependencies, and
planning/request/phase APIs have no executor/provider/connection/callback input or
I/O call site. Retained migration tests also use an independent empty recorder.
This is not a simulated real-server rollback or an instrumented provider that was
called and refused. Live provider0/DDL0/DML0/apply0/COMMIT0 follow the closed
statement/call graph and fixed event register; no provider/DDL dispatcher exists.

New real suites use limited quent_test and hidden metadata roles in the dedicated
namespaces. Setup DDL/DML/termination and independent observer SQL are test-only,
not a product apply API. An independent connection verifies before/after catalog
column count/name/native/nullability/default and retained data. Additional fixture DDL deliberately creates drift,
nonunique indexes, nullable/default-NULL ambiguity and unsupported metadata.
Two tables exercise nine completed SELECTs and exact sorted scope; no table order
normalization changes the private migration plan's declaration order.

Both dialects measured the following **product** event counts per fresh reader:

| Scenario | Catalog SELECT attempt/complete | BEGIN attempt/ACK | Explicit ROLLBACK attempt/ACK | Connection |
| --- | --- | --- | --- | --- |
| one-table positive | 5/5 | 1/1 | 1/1 | 1 |
| two-table positive | 9/9 | 1/1 | 1/1 | 1 |
| query budget 2 | 2/2 | 1/1 | 1/1 | 1 |
| row budget 1 | 2/1 | 1/1 | 1/1 | 1 |
| string budget 1 | 1/0 | 1/1 | 1/1 | 1 |
| role mismatch / live default drift / nonunique index / nullable default NULL / unsupported metadata type | 5/5 | 1/1 | 1/1 | 1 |
| missing role visibility | 3/3 | 1/1 | 1/1 | 1 |
| fixture downstream disconnect after header | 3/2 | 1/1 | 1/0 | 1 |
| cancel after first Pending | 0/0 | 1/0 | 0/0 | 1 |
| before-BEGIN target/TTL refusal | 0/0 | 0/0 | 0/0 | existing reader |
| unpolled future | 0/0 | 0/0 | 0/0 | existing reader |

An attempt is the adapter event before waiting on SQLx, not proof that every byte
was sent on wire. Complete is end-of-result acknowledgment. Successful complete
SELECTs are real DB measurements. Handshake/SQLx protocol control is outside these
catalog counters. In the final nanosecond-timeout trials MySQL reached SELECT attempt/complete
1/0 and PostgreSQL 2/1, both with BEGIN 1/1 and explicit ROLLBACK 0/0. These
counts are scheduling-dependent and not asserted as deterministic wire boundaries. Timeout/error/cancel returned no evidence and quarantined the reader.
Fixture disconnect uses one separately counted test hook; PostgreSQL terminates
its own backend, MySQL uses a fixture CONNECTION_ID SELECT plus KILL. Those are not
product catalog SELECTs. Best-effort Drop/close is not counted as ACK rollback.

Real BEGIN failure and isolated lost-ROLLBACK-response packet faults were not
injected. Real downstream connection termination demonstrates cleanup without ACK,
not an independently isolated rollback packet-loss control. Synthetic private clock
and malformed-result tests are distinguished from real catalog observations. Old
cancel/ordinary UPDATE response-holding controls remain in full regression.

## Compatibility and remaining unsupported scope

No new dependencies/features or external input format. Rust 1.94, edition 2024,
publish=false, exact SQLx/Tokio/other pins and Cargo.lock stay unchanged. Core has
zero normal dependencies; default facade/CLI have only Core. Existing source wire,
manifest fingerprints, CRUD/public output and protected export behavior remain.
The new output-only view has an independently checked exact allowlist. Compiler
harness retains 347 negative + 19 positive controls and adds 14 + 1, yielding
361 diagnostic-checked negatives + 20 positive controls. Existing generated
consumer compiler controls are unchanged and rerun with full/feature regressions.

Before native types are signed integers only. Cross-namespace observation,
backfill/contract work, index/FK/CHECK definitions, full-effects completeness, source
spelling of absent versus NULL, physical identity/TLS/permissions beyond the finite
checks, native Windows live/ACL, continuous monitoring and post-observation DDL
exclusion remain unsupported/unverified. Caller-declared exclusion is bound and
rechecked, not independently authenticated. PG's finite catalog snapshot is not a
DDL lock; MySQL information_schema is not promoted to a PG MVCC snapshot. NoReady/
Partial evidence cannot satisfy a future full-schema apply gate. Migration locks,
conversion/data loss and PG/MySQL apply/rollback semantics remain outside PR2.

Exact command results, failures/filter selections, final settings/cleanup and hosted
CI are recorded below after local verification and in the execution report. No
historical PR43 failure or prior maintenance CI result is reused as this head's CI.

## Local regression commands and distinctions

Final dedicated WSL full run: `scripts/test-dispatch.sh cargo test --workspace
--all-features --locked --offline -- --test-threads=1 --nocapture` passed **257**
tests, **0 failed / 0 ignored / 0 filtered**, across 28 suites. This retains the
252-test baseline and adds one Core phase test, one facade private-boundary test,
two real preflight suites and one compile-fail doctest. An earlier full run also
passed 257; the final runs include additional multi-table/metadata-type, return-boundary
total-timeout and exact independent observer metadata checks. Core's focused library run passed 17;
focused pure/live selections passed 1/2 respectively but excluded other tests and
are not presented as full regressions.

`cargo test --workspace --locked --offline -- --test-threads=1` passed **114**
(default baseline 113 + phase test), with zero failed/ignored/filtered. Approved
DB-free all-feature `--skip live_` passed **237**, zero failed/ignored, 20 substring
exclusions. The three incidental DB-free exclusions were independently run with
package/test-binary/`--exact` selection and each passed one: manifest public live
claim nonpromotion, CLI live-config rejection's actual detecting connection
control, and Core current-context nonpromotion. Their targeted selection filters
are 9/5/4 respectively; these exclusions are not counted as additional successes.
The remaining 17 exclusions are real live tests covered by full. No skip/ignore/
0-selected substitute is used for live acceptance.

Initial sandbox DB-free selection failed three existing loopback positive controls
with bind PermissionDenied. Approved ephemeral loopback bind/close diagnosis
succeeded; approved rerun passed all 237. An incorrect initial incidental selector
also selected eight real DB suites without disposable-DB environment; those eight
failed preconditions and are not counted as success. Selection was corrected to
three exact names; the same eight live suites had already passed in final full.
Neither failure was bypassed through an ignored test or altered product rule.
Initial development compile errors (private macro access, temporary borrow, helper
borrow and a Result helper unavailable in Rust 1.94) were corrected before final
validation; these failed builds are not acceptance evidence.

Both final DB runs measured PostgreSQL **16.14**, UTC, default read committed,
UTF8, en_US.utf8 (reader uses explicit repeatable-read read-only), and MySQL
**8.4.8**, +00:00, REPEATABLE-READ, utf8mb4/utf8mb4_bin with
ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,
ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION. Existing FOUND_ROWS/InnoDB,
GIPK visibility/case checks and limited roles/namespaces are retained. Images:
`postgres@sha256:16bc17c64a573ef34162af9298258d1aec548232985b33ed7b1eac33ba35c229`
and
`mysql@sha256:2952e3be7807f06fc18de50b3ea1a632d5c70d63482ff7d7376fe3aa8999babf`.
Only new dedicated fixture containers were used. Each run removed exactly its
containers/anonymous volumes and verified their absence, including failures' cleanup
paths. No existing application DB/global setting was modified.

`cargo fmt --all -- --check`, workspace default/all-feature builds, warning-denied
all-target/all-feature Clippy and rustdoc passed. Core/default facade/default CLI
normal dependency trees were checked separately. `scripts/check-features.sh
--offline` passed all **262 cargo check invocations** and its **3 CLI test commands**.
All **30 facade + 8 CLI** configured feature combinations passed test and
warning-denied Clippy as **76 sequential commands**. The verification runner
completed 90 top-level commands successfully (including those 76 and the feature
script). Empty feature-gated test binaries are not counted as runtime case evidence;
the full run supplies actual selected runtime cases. The unchanged generated
consumer harness reran its 76 diagnostic negatives + 8 positive controls in six
invocations per harness run. All heavy compiler/build/test commands were sequential.
Hosted Ubuntu/Windows jobs and actual PR head are separate execution-report evidence.
