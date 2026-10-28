# QT-AI-12 PR2 finite local evidence

Work `8bd0b8b7-6df6-40ca-9501-6fef1b24d5b4`, revision1, public execution
`db11b5b9-6e74-40c6-9116-ab82072cd263`. Specification consultation
`914abf4f-f10b-4272-b9d4-0795bc8f68e6` delivered the non-Tx-SQL counting contract A.
This evidence accompanies [the finite contract](query-observation-v1.md), not all
24 full acceptance cases. Exact committed head, PR URL and per-job hosted results
belong to this execution's RelayWeft report; this file makes no future-head claim.

## Environment and preconditions

Actual cwd `/home/murai/github/quent`; WSL2 Linux6.18.54.1, distribution Ubuntu,
Ubuntu24.04.5; Rust1.94.0. No applicable AGENTS.md found in repo or ancestor paths.
Origin `git@github.com:recoweft/quent.git`; initially clean PR1 source branch at
87e52a87e3e5e2bf73754bf0740aff02a036038e. Fetched origin/main
 d852f1321c058bc99e25988dfdea96067e736a9f has the identical tree. GitHub API
confirmed PR48/24/26/28 merged into main at the supplied SHAs; all are ancestors
of the fetched main. Current Issue1/13 bodies were retrieved; both comment lists
were empty. SSH authenticated as recoweft (GitHub's no-shell exit1 is expected);
`gh api user` returned recoweft and repository permissions included push/admin.
These read-only checks alone do not prove an actual push or PR creation succeeded.
The requested feature branch was created from fetched origin/main after checks.

Sandbox DNS, .git writes, Docker socket and socket creation were restricted.
Narrow approved read-only remote/account checks, fetch, branch creation, Docker
version/local-image checks and loopback listener/connect/accept diagnostics succeeded.
No host/global/network/database policy was changed. Each later remote mutation or
fixture execution still uses the tool approval mechanism. No Windows program or
uncached dependency acquisition is needed for this local implementation; Windows
live DB/SDK/ACL tests and actual TLS remain unverified.

Both real DB runs use only `scripts/test-dispatch.sh`'s **new** loopback containers
and limited quent_test namespace role, never DATABASE_URL or production services.

| Engine | Actual settings / digest |
| --- | --- |
| PostgreSQL16.14 Alpine x86_64 | UTC, read committed, UTF8, en_US.utf8; postgres@sha256:16bc17c64a573ef34162af9298258d1aec548232985b33ed7b1eac33ba35c229 |
| MySQL8.4.8 | +00:00, REPEATABLE-READ, utf8mb4/utf8mb4_bin; ONLY_FULL_GROUP_BY, STRICT_TRANS_TABLES, NO_ZERO_IN_DATE, NO_ZERO_DATE, ERROR_FOR_DIVISION_BY_ZERO, NO_ENGINE_SUBSTITUTION; mysql@sha256:2952e3be7807f06fc18de50b3ea1a632d5c70d63482ff7d7376fe3aa8999babf |

SQLx's pinned FOUND_ROWS count semantics and fixture InnoDB tables remain; no
changed-row reinterpretation. Each script's EXIT cleanup removes **its own**
containers/anonymous volumes and checks absence, including failed trials. These
versions/settings are WSL fixture evidence, not extrapolation to Windows.

## C21-centered production observations (each engine)

| Case | Business submission attempts | Other calls / consumption / outcome |
| --- | --- | --- |
| Limits0/1/2 | respectively0/1/2 successes, then rejected next SQL0 | No auxiliary/EXPLAIN/provider or owned Tx; prior sends remain charged |
| New request, same context |1 | repeat_count1 independent of prior request; no refund/reset of old request |
| Repeat same structure, changed literal/limit values |2 | repeat_count1 then2; threshold2 suspicion only on second; second result may be empty but fully consumed |
| Changed projection |1 | separate shape, repeat_count1; no suspicion |
| Changed target/tenant/subject snapshot on attachment |0 | ContextMismatch before dispatch; no Tx/provider |
| Entry capacity1 with SQL budget4 |1 then next0 | capacity/incomplete/closed; no silent observation truncation |
| Created/dropped unpolled validated future |0 | no entry, handle consumed |
| Final expiry equal to exclusive boundary |0 | Refused; **private clock seam**, no real-time scheduling claim |
| Per-operation ceiling1ns |1 successful complete read | CeilingExceeded/incomplete/closed; no cancellation or rollback claim |
| Baseline versus selected collector |1 each | exact retained SQL/native args and recorder events equal; additional collector I/O0 |
| Compiled OperationSpec read in owned scope |1 | complete result; same common dispatcher |
| Forgotten scope, finish, retained session execute |0 additional | closed request; no post-finish mutation of final evidence |
| Owned exact1 success |1 | begin1/ACK1, commit1/ACK1, Complete/Committed |
| Owned expected0 versus actual1 |1 | begin1/ACK1, rollback1/ACK1; independent observer retains earlier committed value, rejects this attempt's different value |
| Owned budget0 |0 business | begin1/ACK1, rollback1/ACK1; earlier committed value persists; no refund |
| Borrowed ordinary read |1 | BorrowedUnconfirmed; library begin/commit/rollback0. Caller setup begin/rollback is separate and unobserved |
| Retained borrowed command execution after forgotten scope |0 business/ledger | Unsupported/NotStarted, closed/incomplete; next ordinary dispatch0; library Tx attempts0, caller retains completion |
| Existing 256-row limit counterexample |1 | 256 completely decoded rows, Partial, failed/incomplete/closed; no truncated success |
| Existing owned begin/commit/rollback errors and lost-success ACK cases |0 for begin failure;1 otherwise | Unknown; **artificial completion seams**, ACKs not fabricated, independent observer distinguishes committed data |
| Existing real row-lock drop/timeout/connection termination |1 each | begin ACK1, no commit ACK or confirmed rollback, Unknown/not-consumed, closed; exact connection server wait observed before cancellation |

The live test compares independent data rows and column/type/nullability/order
catalogs before/after, including restoring the one deliberate successful write.
The shared final-boundary recorder measures submission attempts, **not** wire packets
or all server statements. Setup/observer SQL, connect/driver initialization and caller
Tx completion are separate. Private exact-shape unit tests additionally vary namespace,
alias, comparison, native NULL and IN arity: artificial structure evidence, not live
server operator evidence. Existing deterministic PR45 authorized cancel/independent
absence/withheld-ACK controls and existing command protocol tests are retained.
Their success does not make authorized/command/EXPLAIN routes supported in a scope.
Those APIs are absent from scopes and diagnostic-checked compile-time refusals;
retained-session entry guards also prevent a forgotten scope from bypassing them.

C01/C06/C08/C11-C19/C23/C24 retain relevant shared validation/policy/tenant/request/
ownership/expiry/Tx/result/output/freshness/input regressions. No parsed public view
becomes context, request, permit or validated operation. Independent JSON key/value
allowlists have a canary-detecting positive control. Compiler totals extend the
retained391 negatives +22 controls to409 negatives +23 controls; generated consumer
retains76 negatives +8 controls in6 invocations. All24 runtime_pending entries remain.

## Paired performance measurements

Debug build, existing production SQLx execution plus bounded test recorders/assertions.
Same already connected per-engine session, same SQL/args/settings/unchanged data and
workload. Four paired warmup rounds, then nine measured pairs, order alternates
baseline-first/collector-first. Each variant round has ten fully consumed SELECTs
and ten exact1 owned UPDATEs with ten begin ACKs and ten commit ACKs:20 business SQL,
auxiliary/EXPLAIN/provider0. Collector construction is outside the span in both variants
(baseline constructs an unattached collector for symmetric setup). Construction/
destruction and public-view readout/serialization are unmeasured; scope
attach/detach, validation, gates, native bind, awaits, complete decoding/consumption
and Tx termination are inside the std::time::Instant span. An extra expected-result
control SELECT is outside measurement. Each variant has260 business submissions
including warmup;180 are in measured rounds. Every round asserts20 actual final
recorder sends and collector-enabled rounds additionally assert20 collected sends.
The original128-entry recorder cap is preserved by using a new bounded recorder
only **after** verifying a completed round.

| Engine / collector | Median ns | Min ns | Max ns | MAD ns |
| --- | ---: | ---: | ---: | ---: |
| PG / absent |18448363|17418203|20633994|1030160|
| PG / present |18784505|14523627|26409916|1079790|
| MySQL / absent |11688767|10816145|17468916|646925|
| MySQL / present |12048480|10785769|16655379|1262711|

Raw measurements in chronological variant order, ns per20-operation round:

- PG absent: [19510108,17418203,20251447,18286685,18049382,17709465,19962689,20633994,18448363]
- PG present: [26409916,21996437,17711591,18784505,19864295,20045503,18211842,17933606,14523627]
- MySQL absent: [15319807,17468916,15251758,11466635,11212276,11688767,10816145,11041842,11854780]
- MySQL present: [16269130,16655379,14850072,12048480,11747589,12011656,10785769,11932563,13978985]

This small noisy local paired sample does not establish an improvement, overhead
bound or Windows performance. Differences vary within observed dispersion/cache/
scheduling/CI noise. No percentage target was assumed before measurement, no
estimated optimizer cost was substituted, and no release benchmark is claimed.

## Regressions and development failures

No new optional feature is added. The existing feature script was enumerated:
272 grid checks +6 continuation checks =278 distinct exact check invocations;
its3 additional CLI test invocations remain. The CI arrays still contain30 facade
and8 CLI feature sets, each tested and warning-denied Clippy checked:76 commands.
Original cases and generation/regeneration checks are retained, not replaced by a
smaller subset. Core/default dependencies and Cargo.lock are unchanged.

Development failures are not acceptance: initial Rust borrow/privacy/borrowed-API
compile errors; test helper visibility; first live trial hit the existing finite
128-send **test recorder** cap during the repeated benchmark (16 passed/2 failed,
35 filtered). Both failed-trial containers and volumes were removed and absence
verified. The recorder assert was preserved; round-local count checks precede
resetting test-only recording. A subsequent full development run passed272 with
0 failed/ignored/filtered. After strengthening final-evidence sealing, exact-value
rollback controls and compiled-read coverage, the final source was rerun separately.
Full final command counts, all failures/exclusions, current-source test/Clippy/doc/
feature results, cleanup, actual push/PR and exact-head CI are recorded in the
execution report. DB-free exclusions and 0-selected configurations are never live
DB evidence or completion of any full runtime_pending case.
