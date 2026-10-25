# QT-AI-08 PR1: finite local measurements

This file records the implemented bounded subcases, not completion of the 24 full
runtime scenarios. All 24 `runtime_pending` fixture markers are unchanged.
The corresponding exact PR head, hosted checks and RelayWeft report receipt are
recorded in the actual PR/execution report, rather than a self-referential hash here.

Environment measured: Ubuntu 24.04.5 / WSL2 Linux 6.18.54.1, Rust 1.94.0.
Origin `recoweft/quent`; branch `feature/qt-ai-08-review-cli` starts at verified
main merge `fb1c9dd9669354d0748e6773074b9e45b2599f4c`. Ancestor merges for PR18/19,
25–30 and 31/32 were read from fetched main. API worker `recoweft`, repository
push permission, SSH push dry-run and Docker 29.8.0 were checked independently of
Issue connection authentication. No ancestor/project AGENTS.md was found in the
checked parent paths or repository file search. Old execution lease handling was
left to the controller; its start answer and plan-wire specification answer were
received through the fixed execution's question workflow.

| Local command / selection | Observed result |
| --- | --- |
| `cargo test --workspace --locked --offline` | 84 passed, 0 failed/ignored/filtered; retained default 81 + two panic-hook harness tests and one finite review grammar test |
| `cargo build --workspace --locked --offline` and `cargo build --workspace --all-features --locked --offline` | passed |
| `cargo test --workspace --all-features --locked --offline -- --skip live_ --test-threads=1` | 173 passed, 0 failed/ignored, 10 name-substring exclusions |
| Core `c12_exact_target_and_context_missing_never_become_live_or_permission`, facade manifest `public_canaries_and_live_claim_nonpromotion`, CLI `live_config_rejection_has_zero_connect_with_detecting_control`, individually | each passed; these three incidental substring exclusions supplement DB-free selection |
| `bash scripts/test-dispatch.sh cargo test --workspace --all-features --locked --offline -- --test-threads=1` | 183 passed, 0 failed/ignored/filtered; 167 retained +16 additions |
| `cargo fmt --all -- --check` | passed |
| `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings` | passed |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked --offline` | passed |
| `bash -x scripts/check-features.sh --offline` | all 130 cargo checks passed (96 retained +32 review overlays +2 cross-feature checks), manifest-only/review-only product CLI tests passed |

All 20 output-feature test/Clippy combinations passed sequentially (16 retained + four review overlays), with warnings denied. Core/default normal dependency trees remain unchanged; standalone manifest/operation-spec/codegen/review normal trees have no driver/runtime.

The full and DB-free runs were sequential, including shared compiler harness,
fresh generated consumer and actual product CLI build/run. Shared compiler passed
179 diagnostic-checked negatives and nine compiling controls (170/eight retained).
Fresh production-generated consumer retained 76 checked negatives and eight controls
in six control invocations. Compile failures assert expected diagnostics and exclude
missing-crate/import failures. These counts describe compiler programs, not extra
workspace test-function counts.

The 16 additional all-feature test functions comprise seven pure review parser/
inspection/output/tracing tests, one explicit typed review exact-current/compiler
regression, two test-only panic-hook subprocess tests and six Unix CLI tests.
The standalone default grammar test checks unsupported-feature and invalid-grammar
refusal without reading paths; it is not a successful review on the default build.

Plan/Spec independent full public expectations are checked both by the library and
real CLI JSON/pretty subprocesses. A valid plan has finite-data SQL/native precision,
High/Unknown from missing facts, exit 3; a missing-WHERE write retains Blocked and
MissingWhere, exit 5. A valid syntax-only Spec has literal-range precision and all
application/runtime axes unverified, exit 3. Manifest domains retain independent
Complete/Unknown/Partial/Unsupported/Missing support and Unverified/Missing freshness;
parsed Unknown/Partial exits 3, Unsupported exits 4. Invalid/budget is 2 and I/O is
8 with fixed diagnostics. There is no review exit 0 or execution/CI-gate acceptance.

Counterexamples exercise external claim tampering/same-fingerprint different data,
old output-only kinds, unknown kind/version/native, duplicate escaped keys,
extra/missing/null, signed/unsigned/NULL/range and byte/depth/array/string budgets.
Public JSON/pretty/Debug/Display/fixed errors have no source canaries. Tracing's
real subscriber sees no implicit review events and has a detecting event control.
The CLI loopback test has an actual TCP connection/accept control, then observes no
connection from valid or grammar-rejected reviews with an ambient DSN. Structural
absence of provider/dispatcher entrypoints is the provider-zero evidence, not a
fabricated transaction trace. Fixed panic hook tests inject a panic only into the
child **test harness** with backtrace/tracing env, not a product environment bypass.

Actual files cover exact 65,536/max+1 bytes, invalid UTF8, writerless FIFO, directory,
/dev/null, final symlink and repeated regular/FIFO/symlink path replacement. Every
new CLI child is watched for five seconds; timeout kills and reaps, then fails.
Existing three verify input regressions are retained. This measures no filesystem-
wide deadline, immutable snapshot, physical target authentication or Windows live DB.

Both DB image digests are unchanged in scripts/test-dispatch.sh. Only new dedicated
containers/catalogs/namespaces/limited roles were used. PG16.14 measured UTC/read
committed default/UTF8/en_US.utf8; the reader's explicit repeatable-read read-only
workflow is unchanged. MySQL8.4.8 measured +00:00/REPEATABLE-READ/utf8mb4/utf8mb4_bin,
ONLY_FULL_GROUP_BY/STRICT_TRANS_TABLES/NO_ZERO_IN_DATE/NO_ZERO_DATE/
ERROR_FOR_DIVISION_BY_ZERO/NO_ENGINE_SUBSTITUTION; retained FOUND_ROWS adapter.
Script cleanup removed its containers and anonymous volumes and verified absence.
Review adds no new write/Tx/live workflow; the full DB suites are retained regressions.

Sandbox DNS, Docker socket, Git metadata write and loopback socket restrictions
were observed; they were not recorded as host/auth failures or successful tests.
Narrow approved SSH/API/Docker/loopback diagnostics succeeded, followed by approved
repo branch setup and requested validation. No approval denial occurred; no global
network/sandbox/Windows/DB setting was changed. Initial development compile failure
(consuming SpecSource through a borrowed argument) was corrected to an owned entry;
it is not counted as passing evidence. Required checks above describe passing runs.

Remaining limits: Rust source PR2; SARIF/diff/baseline/suppression PR3; no new provider,
permission, write-spec, command, migration, EXPLAIN, publishing or sensitive exporter.
Trusted application authentication/assertion accuracy, actual runtime correspondence,
physical server identity/replacement, general DDL races, TLS, Windows live DB,
Windows export/ACL success and arbitrary OS I/O deadlines remain unverified or
unsupported. Native hosted Windows/Ubuntu checks must be read on the actual head;
WSL evidence alone does not establish them. No merge, Issue close or release occurs.
