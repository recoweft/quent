# QT-AI-11 PR3 finite evidence

Work `5c3b4d7c-4742-4836-814a-f4cc7f34563c`, revision 1, public execution
`28b2d512-12fb-4013-afec-9ee1692ad57a`. Scope: only the
[explicit finite application contract](migration-apply-v1.md). The separate
RelayWeft report records exact final head, actual PR URL and CI API evidence.
No prior work's report is reused as this work's receipt. Issue12 and all 24 full
runtime_pending markers remain open/pending. No merge, Issue close, release,
crates.io publish or production-user DB application is part of this work.

## Checked identity and prerequisites

Actual cwd `/home/murai/github/quent`, Ubuntu24.04.5/WSL2, Rust1.94.0 matching the
pin. No applicable ancestor/repository AGENTS.md; initial tree clean on PR2 source
`2b8b473a87ee58096753ba8e52413160f0fc5f7f`. SSH fetch/push origin is
`git@github.com:recoweft/quent.git`. Actual remote main was read and fetched as
`8d38d3d31e5e013af51fdf52c346195b50007d75`, then used to create exactly
`feature/qt-ai-11-migration-apply`. PR2 source and required dependency merges
`4d3c7e3832543af8e9c72f30814c5316e938db71` and
`26377bdada75d5a57a6a87ad35cf9d4ec28bbdba` were independently checked as fetched
main ancestors. Worker-authenticated API read Issue1/12 in full, PR46 merged/main
information, worker identity recoweft and repository push permission. Specified
branch SSH push dry-run succeeded. Actual push and PR creation are separate
report/PR evidence, never inferred from Issue connector credentials.

Sandbox DNS and Docker socket access initially failed; sandbox gh status also
reported unusable authentication during that failure. Narrow approved read-only
diagnostics succeeded for GitHub SSH/main and worker REST authentication/push
permission, plus Docker29.8.0. Approved fetch/branch creation and disposable DB runs
succeeded. No DNS/network/firewall/WSL/global policy/config change was made.
No Windows process launch/SDK/live DB/ACL check was performed by this WSL worker.
Hosted Windows DB-free CI, if successful on the actual PR head, is distinct evidence.

The controller-delivered answer to question
`d8b53a8a-cc48-4cdd-9c03-929f96c6e6da` was read with that exact ID after delivery
notification. Its A boundary is applied in this same revision: separate dedicated
apply connection, new same-session finite observation, five independent application
assertions, separate human decision and independent synchronous current snapshots.
It does not promote PR2 Partial evidence or claim a working external human identity
provider, physical authentication implementation or exclusion service.

## Cases and sources of evidence

| Cases | Finite implementation/test | Limits |
| --- | --- | --- |
| C01/C22 | Candidate and request construction, original planner/compile/review remain DB/provider zero | Structural closed call graph, not an invoked provider that refused |
| C08/C19/C22 | Old view/Fresh/manifest/reader/evidence/CRUD permit cannot become apply authority | Real compiler negatives plus compiling new API control; no new JSON input |
| C12/C14/C22 | Exact plan/target/context/subject/session/request/registration/phase/visibility/exclusion and assertion generation/expiry | Independent synchronous source and real clock; replaying stale source facts violates the application contract |
| C13 | Linear private candidate/decision/handle; future consumes ownership even unpolled | Real compiler move/Clone/serde/private-field negatives; no exactly-once assertion |
| C15/C23 | Redacted sensitive Debug, fixed output-only JSON allowlist, closed error/source None, actual product driver tracing capture | Fixture setup/observer output excluded from product capture; app callbacks own their own logging/panic behavior |
| C16/C22/C24 | Both DBs execute original two-step finite DDL and independent observer checks native/order/null/default/data | No schema completeness or all-effects proof is added |
| C12/C14/C22 | Current change after provider/first DDL ACK/PG BEGIN; guard rechecks before each SQLx driver resume | Pure controlled future is a seam; real DB current mutation cases and wire-response controls are separate |
| C22/C24 | Actual duplicate-column server error after first ACK; constrained-table/MyISAM/drift refusal | Fixture deliberately violating asserted exclusion tests failure handling, not a production race guarantee |
| C14/C22/C24 | Real table-lock wait cancellation and expiry, retained quarantined progress | Observer snapshots after drop/unlock do not prove client ACK or universal termination/rollback |
| C16/C22 | Actual SQLx/server protocol ALTER/COMMIT ACK withheld on bounded plaintext loopback proxy | Actual server CommandComplete/OK seen by fixture, never forwarded to SQLx; not a mock response or general network-fault model |

## Real product statement/response boundaries

All positive captures use a new connection and a new finite observation on that
connection: 5/5 SELECT attempt/complete, observation BEGIN1/ACK1, observation
ROLLBACK1/ACK1, separate migration adapter1/response1. No old reader identity is
reattached. The unchanged reader still has no write/COMMIT/apply entry point.
Setup DDL/DML, lock control, test termination and independent observer are counted
separately in test output; they are not product statements/provider calls.

| Scenario | PG apply | MySQL apply |
| --- | --- | --- |
| Two-step positive | BEGIN1/ACK1, DDL2/ACK2, COMMIT1/ACK1; Committed | No apply BEGIN/COMMIT/ROLLBACK; DDL2/ACK2; implicit prefix2 of2 |
| Current changes after first ACK | DDL1/ACK1, unsent1, ROLLBACK1/ACK1; RolledBack | DDL1/ACK1, unsent1; persistent prefix1 |
| Current changes after owned BEGIN ACK | BEGIN1/ACK1, DDL0, ROLLBACK1/ACK1 | No owned apply BEGIN exists; this PG-specific marker is inapplicable |
| Second step actual server rejection | DDL2 attempts/ACK1/rejected response1; ROLLBACK1/ACK1 | DDL2 attempts/ACK1/rejected response1; prefix1 may persist; StepRejected |
| Before-DDL expiry/current/authority refusal | DDL0 | DDL0 |
| Lock-wait cancel after driver attempt | DDL1 attempt/ACK0, unknown step0, unsent1, no rollback ACK | Same unknown step/prefix0/suffix1; no rollback promise |
| Lock-wait deadline | DDL1 attempt/ACK0; ROLLBACK attempt without ACK, RollbackUnknown | DDL1 attempt/ACK0, StepUnknown |

Attempt is an adapter/driver-call boundary, not proof every byte was sent. The
polling guard can refuse before resuming SQLx. Concrete positive ACKs prove the
measured statement completion; the proxy additionally counts actual forwarded DDL
Execute packets. Handshake/preparation and protocol traffic are outside catalog
SELECT counters. The shared observation budget remains PR2's finite ceilings.
No CRUD cardinality rule is used for DDL: real successful ordinary ALTER
rows_affected=0 and COPY ALTER rows_affected=1 are both accepted. The COPY case sets
old_alter_table=ON only on the dedicated MySQL session through a test-only fixed
setup statement (one separate fixture session-control count), then executes the
unchanged original private SQL twice. Both real responses report1; independent
observer checks three columns, native/order/null/default and data7/new NULL values.
The production library exposes no session-control method.

Per DB, the general suite separately exercises provider-response/current mismatch,
each of five Unknown assertion responses, independent human deny, wrong invocation,
missing adapter/subject, each missing current assertion, unknown human instruction,
changed subject/session/plan/phase/role visibility/exclusion/request/registration,
and a freshly allocated operation carrying the same textual request ID,
expired approval, wrong target, real default drift and real nonunique index. MySQL
also exercises unsupported MyISAM. These refuse before any product DDL. Pure
Refused/destructive plans cannot construct the preflight/apply request; baseline
migration tests retain drop/type/null/default/unknown/unsupported all-SQL-zero cases.
No adapter can waive an observed unknown/default ambiguity or unsupported structure.

Independent observers verify column names, signed native types, ordinal order,
nullability/defaults, retained value7 and new-column NULL data in positive and
partial/error cases. The second-step rejection fixture first appends `tail_two`
**outside** product apply: PG rollback removes product `tail_one` but preserves that
fixture column; MySQL preserves fixture `tail_two` followed by ACKed product
`tail_one`. This is not evidence that PG rolls back fixture/external changes.

## Actual protocol ACK withholding

A test-only bounded plaintext loopback proxy parses real PG frames and MySQL
prepared Execute/OK framing, forwards preparation normally, and withholds the
selected server completion plus subsequent responses. It logs no SQL or payload.
The SQLx production adapter receives no fabricated success. An independent observer
uses a separate original connection, outside the proxy.

| Real withheld response | Client result | Independent catalog snapshot |
| --- | --- | --- |
| PG first ALTER CommandComplete | DDL ACK0, unknown step0/suffix1, RollbackUnknown; no cleanup ACK | Before columns1 visible while uncommitted; data7 after connection close |
| PG COMMIT CommandComplete after two DDL ACKs | CommitUnknown, DDL ACK2, no auto retry | Columns3 already visible despite no client COMMIT ACK |
| MySQL first ALTER OK | ACK prefix0, unknown step0, suffix1 | Columns2 already applied |
| MySQL second ALTER OK | ACK prefix1, unknown step1, suffix0 | Columns3 already applied |

All four controls assert actual server ACK seen and withheld, and real DDL Execute
count1/2. Observer also checks native/order/null/default/data. Socket drop is not a
client ACK. This finite real protocol fixture does not cover TLS, packet corruption,
all disconnect races, failover, crashes, distributed routing or arbitrary drivers.
Real apply BEGIN failure and isolated lost-ROLLBACK-ACK packet are still unmeasured;
PR2's original such unverified items remain unverified. No general failure model,
exactly-once, full rollback, lock freedom or data-loss guarantee is claimed.

Private `ApplyRequest` is non-Clone/non-Copy/non-serde. Independent current supply
retains that operation allocation as well as its complete bounded textual ID.
Reconstructing the same ID does not restore the operation; the real DB suites
refuse that changed allocation after successful observation/approval and before DDL.
The same post-seal DDL0 cases cover subject/session/plan/phase/visibility/exclusion/
textual request/registration/expiry/target changes; they do not merely test admission. This follows the
existing CRUD request-identity discipline without reusing its DDL-ineligible permit.

## Regression, settings and development failures

Final command counts/results and exact head/CI are recorded in this revision's
RelayWeft report after validation. All heavy build/compiler/test commands run
sequentially. Focused selections report filters and never replace full acceptance.

Dedicated fixture images are unchanged:
`postgres@sha256:16bc17c64a573ef34162af9298258d1aec548232985b33ed7b1eac33ba35c229`
and
`mysql@sha256:2952e3be7807f06fc18de50b3ea1a632d5c70d63482ff7d7376fe3aa8999babf`.
Observed PG16.14/UTC/read committed default/UTF8/en_US.utf8 (observation explicitly
repeatable read read only); MySQL8.4.8/+00:00/REPEATABLE-READ/utf8mb4/utf8mb4_bin,
ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,
ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION. Existing FOUND_ROWS options,
InnoDB/visibility/case checks, limited roles and explicit namespaces are retained.
Only new per-run containers are used. Every completed run, including failing runs,
removes its containers/anonymous volumes and verifies absence; no global prune,
production DB application, GRANT recovery or application configuration bypass.

Development failures are preserved, not counted as success: initial macro sibling
privacy/type alias visibility checks; test capture helper/type inference build
errors; initial live product-capture assertion included fixture notices (both
suites failed); the next run passed PG but MySQL LOCK TABLES fixture failed because
that command is unsupported in prepared protocol. The fixture now uses test-only
raw control, with product capture checks immediately around product calls. A later
Clippy/build attempt caught instrumentation format/type inference issues and then
identical network-parser branches; these were corrected before final checks.
The final product has no raw control/SQL callback API. No ignored test, altered DB
permission, network policy change or changed dependency pin bypassed these failures.

Core/default dependency trees, source/command/authorization/owned/borrowed contracts,
compiler controls, frozen CLI interfaces, feature isolation and output exports remain
regression requirements. Actual external human authentication, physical server/TLS
attestation, complete effects/provenance and exclusion mechanisms remain application
responsibilities rather than newly measured facts. Native Windows live DB/ACL and
all 24 whole-system runtime_pending cases remain unverified/pending.

## Final local validation results

Final full command: `scripts/test-dispatch.sh cargo test --workspace --all-features
--locked --offline -- --test-threads=1 --nocapture`: **263 passed, 0 failed,
0 ignored, 0 filtered**, 28 suites. Retains all257 and adds two pure/current/output
controls, two general real apply suites and two actual protocol-loss suites. The
final tests include post-seal target/plan/subject/actual-other-session/request/
registration/phase/visibility/exclusion/expiry changes and observer-confirmed DDL0.
Compiler harness: **377 diagnostic-checked negatives + 21 compiling controls**
(361+20 retained, 16+1 added). Generated consumers retain **76 negatives + 8 positive
controls in six invocations**; these internal compiler counts are not added to the
263 runtime/doc/harness test total.

`cargo test --workspace --locked --offline -- --test-threads=1`: **114 passed**, no
failure/ignore/filter. All-feature DB-free `--skip live_` selection: **239 passed**,
no failure/ignore, **24 substring exclusions**. Three incidental DB-free exclusions
were independently selected with package/test-binary/`--exact`: manifest public
live-claim nonpromotion, CLI live-config rejection with detecting loopback control,
Core exact current-context nonpromotion: **1 pass each**, selection filters9/5/4.
The other21 exclusions are actual live tests and all ran in full. Filter/skip/
empty feature-gated binaries are never counted as selected live case success.

fmt, workspace default/all-feature builds, all-target/all-feature warning-denied
Clippy and warning-denied rustdoc passed. Core normal tree has zero dependencies;
default facade and CLI normal trees each contain only Core. Cargo.lock, all Cargo
manifests and toolchain pins are unchanged. Feature script passed **262 cargo check
invocations and three CLI test commands**. All **30 facade + 8 CLI** configured
feature combinations passed tests and Clippy as **76 sequential commands**.
The verifier completed **90 sequential top-level commands**. After the final
private-request and post-seal fixture refinements, **14 affected verification
groups** (including all-feature regression/compiler, builds, Clippy/doc and the
entire262-check script) were rerun and passed; the 76 driver-free feature commands
exercise unchanged configurations in which the new apply module is not compiled.
The final full263 run includes the final product and strengthened fixture code.
Earlier calibration full runs260/263 and focused selections are superseded by this
final result rather than summed into a larger acceptance count.

All ten local dedicated-DB runs (six full runs and four focused live runs,
including the two initial failing focused runs) printed successful per-run
container/owned-anonymous-volume removal and absence verification. No production
DB was used. In the final lock-wait observer snapshots, PG cancel and timeout each
showed one original column; MySQL cancel showed one, timeout showed two. Value7,
native/order/null/default were retained. These timed snapshots are observations,
not client completion ACKs or a general cancellation outcome guarantee.

SSH `ssh -o BatchMode=yes -o ConnectTimeout=10 -T git@github.com` independently
identified recoweft as successfully authenticated (GitHub's normal no-shell exit1).
Before PR creation remote main was still the expected8d38d3d; PR43/45/46/26/28
were confirmed merged via worker API, their relevant ancestors checked, specified
remote apply branch absent, and Issue12 still OPEN. Actual final commit/push/PR
and hosted CI are the separate execution report's final API evidence.

Final review removed an inappropriate exact-zero result guard before the final
follow-up commit. It would conservatively misclassify a successful COPY ALTER's
positive copied-row count as unknown. The explicit real MySQL COPY response control
now detects this case (two real ACKs, rows_affected1 each), in addition to the zero
responses retained by both dialects. Its initial focused selection passed one
with48 filters; it is included in the final unfiltered full263 rerun. Earlier
head5078eb2 and its CI runs are superseded by the actual final PR head recorded
in the execution report; they are not substituted for final-head CI.
The COPY follow-up's final full263 rerun passed with0 failure/ignore/filter and
verified dedicated cleanup. All14 affected verification groups were rerun again
after that change and passed, including all262 feature checks, final114 default,
239 DB-free plus the three exact incidental controls, Clippy and rustdoc. The
same final-head CI requirement applies to the follow-up commit. Its test-only
session setup is absent from the production build; no public/raw driver route was
introduced by measuring COPY responses.
