# Reproduce the finite repository workflows

Run from the repository root on the exact source commit being reported. Rust
1.94.0 and edition2024 are fixed by the toolchain/workspace. Normal Rust dependencies
are locked in Cargo.lock; first use requires registry access, or a previously
populated cache. Standalone generated consumer controls resolve offline against
that cached locked set, so CI explicitly runs `cargo fetch --locked` for codegen
matrix jobs. Missing dependencies/environment is a failure or unverified check,
not a skip that demonstrates acceptance. Heavy compiler/build commands run sequentially.

These procedures use existing production examples and regression tests; they add
no production executor or artificial docs-only mirror tests. Commands without
`--offline` may download missing locked dependencies. With a complete cache, adding
`--offline` to cargo commands gives the equivalent local dependency selection.
Commands below expect exit0 except the explicitly described CLI refusals.

## DB-free introduction

```sh
rustc --version
cargo build --workspace --locked
cargo test --workspace --locked
cargo run -p quent --example tenant_review --no-default-features --features codegen,review --locked
```

Expected toolchain: rustc1.94.0. Default tests run no DB suites and no enabled
all-feature compiler harness. On the PR51 source, Ubuntu default baseline was 114
passed; Windows hosted default was 113. These are **baseline measurements**, not
proof of a new source head. Fresh PR2 commands/counts are recorded in
[PR2 evidence](adoption-evidence-v1.md) and the exact-head PR/RelayWeft report.

The example exits0 after assertions and emits **six** public JSON lines:
regeneration `no_change`/changed_domains0; patch `inspection_only`/assignments3;
patch inspection review Unknown; read inspection review Unknown; manifest review
Unknown with separate schema/policy/generated-code/live slots; PostgreSQL
migration SqlPlanned/High/steps1/live Unverified. No source/SQL/canary/credential
belongs in those lines. It uses checked synthetic assertions and a fixture model,
not production authentication, least privilege, preflight, human approval or apply.
There is no DB setup or persistent file output to clean up for this example.

## Builds, native features and documentation

```sh
cargo check -p quent --no-default-features --features postgres,runtime-tokio,tls-rustls --locked
cargo check -p quent --no-default-features --features mysql,runtime-tokio,tls-rustls --locked
cargo tree -p quent-core --edges normal --locked
cargo tree -p quent --edges normal --locked
cargo tree -p quent-cli --edges normal --locked
cargo tree -p quent --no-default-features --features mcp --edges normal --locked
cargo tree -p quent-cli --no-default-features --features mcp --edges normal --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

Core has no normal edges; default facade/CLI only Core. MCP-only adds review/parsing
and locked transitives, no SQLx/Tokio driver/runtime. Driver checks are compilation,
not proof of TLS handshake, live DB or Windows local execution.
For warning-denied rustdoc on Bash/WSL:

```sh
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked
```

CI supplies RUSTDOCFLAGS via a job environment on both hosted OSs. No global host
or shell setting needs changing.

## Offline CLI and MCP

These use checked-in finite source files, never public-view JSON as executable input:

```sh
cargo run -p quent-cli --no-default-features --features review --locked -- review plan --json fixtures/review-v1/select.json
cargo run -p quent-cli --no-default-features --features source-review --locked -- review rust --json fixtures/source-review-v1/filter.rs
cargo run -p quent-cli --no-default-features --features manifest --locked -- manifest verify --offline fixtures/manifest-v1/postgres.json fixtures/manifest-v1/postgres.json
cargo run -p quent-cli --no-default-features --features manifest --locked -- manifest verify --offline fixtures/codegen-v2/postgres-source.json fixtures/codegen-v2/postgres-source.json
cargo run -p quent-cli --no-default-features --features review-ci --locked -- review-ci --json fixtures/review-ci-v1/empty.json fixtures/review-ci-v1/empty.json fixtures/review-ci-v1/baseline.json
```

Review has **no exit0**: valid unknown/partial analysis is not a CI acceptance signal.
The SELECT fixture review is Unknown (exit3); Rust syntax fixture review is Partial
(exit3), not resolved type/runtime proof. The identical manifest-v1 fixture comparison
is Unknown (exit3), live Unverified: its incomplete/claimed domains cannot establish
offline success merely by matching themselves. In contrast, the complete supported
codegen-v2 source self-comparison is OfflineConsistent (exit0), live Unverified;
it still establishes no authority. Empty review-ci is Incomplete
(exit3), not proof of complete Git-head coverage. Invalid review input exits2;
unsupported exits4, refused classification exits5 and input/output I/O exits8.
See [review grammar/exits](review-v1.md), [source coverage](source-review-v1.md),
[CI baseline limits](review-ci-v1.md), [manifest CLI](manifest-verify-v1.md).
Do not run these nonzero analysis commands in an unconditional `set -e` sequence
and then report the unexecuted remainder as checked.

```sh
cargo run -p quent-cli --no-default-features --features mcp --locked -- mcp --stdio
```

Use a newline-delimited stdio client, wait for each response before the next request,
and close stdin to finish. Initialize with protocolVersion `2025-11-25`, **empty**
client capabilities `{}` and clientInfo containing only string name/version; then
send `notifications/initialized`. Only `quent_plan`, `quent_compile`, `quent_review`,
`quent_manifest_status` and four fixed `quent://public/*` resource URIs exist.
A different bounded version string negotiates the implemented constant; the client
must disconnect if it cannot support that version. No SDK/all-client compatibility
or remote URL discovery is verified.

Use innocuous IDs: strings <=128 UTF-8 bytes or integer tokens within
±9007199254740991; no float/null/negative-zero, reuse or pipelined requests. IDs
reflect only in top-level response `/id`, including admitted errors; they are not
operation IDs/authority. Profile limits include frame/response65536 bytes,
source4096 bytes, 256 accepted requests and 512 total frames. The
[protocol contract](mcp-v1.md) gives the complete fixed URI/method/budget/error table.
Ordinary CLI typed plan/compile refuses `missing_application_context`. Successful
registered fixtures use application Rust carriers and the same server/transport;
they do not install general product registration. No MCP DB CRUD/Raw/apply/permit/
live catalog/EXPLAIN/source export entry exists.

MCP EOF exits0; Invalid2/Budget3/Timeout6/Io8 are distinct. Deadline checks are
coordinator scheduling checks, not synchronous computation preemption or OS hard
I/O deadlines. A host using `serve_stdio()` must promptly exit its process after
return using Termination's exit code; no second session/background reuse.
The real child protocol suite, including registered/unregistered, zero-connect
controls, cancellation and kill/reap ownership, runs in:

```sh
cargo test -p quent --no-default-features --features mcp --test mcp_stdio --locked
cargo test -p quent-cli --no-default-features --features mcp --locked
```

The facade child suite needs permission to bind and connect a temporary loopback
socket for its independent network detector's positive control. DB-free does not
mean socket-free. A sandbox refusal there stops the suite and is not a passing
zero-connect result; obtain the narrowly scoped tool approval before rerunning.

## Complete finite regression and dedicated DB fixtures

The complete regression includes compiler controls, fresh generated consumers,
public output, real child CLI/MCP and both database engines. Build the CLI and tests
**before** the fixture script's 240s test lifetime:

```sh
cargo build -p quent-cli --all-features --locked
cargo test --workspace --all-features --locked --no-run
bash scripts/test-dispatch.sh cargo test --workspace --all-features --locked -- --test-threads=1 --nocapture
```

Prerequisites: Linux Bash/coreutils (`timeout`), Docker daemon permission, cached
pinned images (or explicit pull access), available loopback port binding and enough
local CPU/disk. The existing [script](../scripts/test-dispatch.sh) creates unique
new loopback-only anonymous-volume containers using PG16.14/MySQL8.4.8 pinned image
digests, dedicated `quent_dispatch_test` role/namespace setup and limited fixture
roles. MySQL retains InnoDB, FOUND_ROWS, strict modes, utf8mb4/utf8mb4_bin,+00:00;
PG retains UTC/read committed/UTF8/en_US.utf8. The script prints actual versions and
settings; record them for each run. Its disposable setup roles are **not** evidence
of production least privilege. It exports only dedicated QUENT_TEST_* connection
configuration to its child and never uses production/DATABASE_URL. Do not copy a
resolved DSN/credential into docs or PR output.

Each run owns its cleanup including failure: EXIT trap removes only its created
containers and anonymous volumes and independently checks their absence. Require
`Dedicated containers and their volumes removed and absence verified` **and** a
successful exit before claiming successful cleanup/test completion. On failure,
record the actual failed phase and any remaining owned resources; do not prune
unrelated Docker data, disable assertions, reuse a production DB or call a missing
fixture a pass. No global DB/host/DNS/firewall/WSL/Codex configuration change is needed.

The PR51 baseline full run was 290 passed/0failed/0ignored/0filtered. It included
425 diagnostic-checked compiler refusals +25 compiling controls and 76 generated
consumer refusals +8 controls across six invocations. Harness totals are not summed
again as distinct Rust test functions; repeated feature executions are separate.
Tenant A/G/M subcase counters and business/provider/auxiliary/Tx/observer boundaries
are specified in [integration acceptance](integration-acceptance-v1.md); actual
measurements belong to the exact source's report. PR45 cancellation/independent
absence/withheld ACK, real command reply-loss and migration assertions remain intact.
No test here completes all 24 original runtime_pending cases.

Explicit DB-free all-feature subset (used by hosted Windows too):

```sh
cargo test --workspace --all-features --locked -- --skip live_
bash scripts/check-features.sh
```

`--skip live_` is substring filtering; it also excludes some DB-free tests. Report
filtered counts; do not treat all exclusions as live DB tests or this command as a
full run. PR51 Ubuntu was264 passed/26filtered; hosted Windows256/25filtered. The
feature script contains312 distinct checks (304 grid+8 continuation) and three CLI
test commands. Repeating the entire local matrix without a relevant change adds no
coverage; preserve it in required CI and verify every final-head job instead.

## CI and OS evidence

[CI source](../.github/workflows/ci.yml) is unchanged. Expected expanded jobset:

| Jobs | Count | Scope |
| --- | ---: | --- |
| workspace | 2 | Ubuntu/Windows default + all-feature DB-free, fmt/build/Clippy/rustdoc/Core tree |
| adapters | 2 | PostgreSQL/MySQL explicit runtime/TLS builds |
| feature-matrix | 1 | 312 checks +3 CLI tests |
| output-features | 68 | 34 facade feature sets ×2 OS, test +Clippy |
| review-cli | 20 | 10 CLI feature sets ×2 OS, test +Clippy |
| Required PostgreSQL and MySQL dispatch | 1 | precompile then pinned both-DB full regression/cleanup |
| Total | 94 | all terminal successful required for this PR's CI claim |

The 34+10 sets mean88 explicit test/Clippy commands per OS collection, not88 jobs.
Record the exact **source head**, run IDs, terminal conclusions and expected jobset;
PR merge-ref tests/old-head green are not current local measurements. Hosted Windows
DB-free is distinct from WSL, Windows local process/SDK, Windows live DB and ACL.
PR51 local Windows stopped before tests due to missing dependencies/native unsigned
script policy; it remains unverified. Do not bypass that refusal using ExecutionPolicy
Bypass, another shell or an equivalent rewritten script. Required environment repair
or approval must follow the active permission rules. No new Windows-local/live/ACL
success is inferred from this docs PR or hosted CI.
