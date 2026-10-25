# QT-AI-08 planned PR2: finite measurements

Work a5e644c1-a359-4a18-8688-6c5ee0340bec revision 1, public execution
c210b93e-a5ca-4946-86fd-8cced5bd972f. Measurements made 2026-10-25/26 JST.
These are finite subcases, not completion of any full runtime_pending case.
Exact PR URL/head and hosted results belong to the actual RelayWeft report/PR;
a report receipt is distinct from PR creation and user acceptance.

## Identity and environment actually checked

Cwd /home/murai/github/quent matched fixed context. Origin SSH remote was
recoweft/quent, WSL2 kernel 6.18.54.1, Ubuntu 24.04.5, rustc/cargo 1.94.0.
No applicable AGENTS.md was found in the repository or checked ancestor paths.
Initial worktree was clean, a single registered worktree and no open repository
PRs were observed. This does not prove absence of every external worker/lease.
Fetched main was e54b8f7f5f2191c970efa77bba12c60b61c2f49b. GitHub confirmed PR33
merged with that merge and head 66ca7b41043c89b6f59fcc57459fbe8c3ef102a8; ancestry
was checked locally. Fetched first-parent history retains dependency merges
PR18/19,25/26,27/28,29/30. The specified feature/qt-ai-08-rust-source-review branch
was created from that fetched main. No alternate work/revision was selected.

Approved SSH ls-remote, API worker identity recoweft, repository push permission
and specified-branch SSH push dry-run succeeded, independently of Issue PAT.
Issue9 comments were fetched (none). Docker server 29.8.0 responded. Sandboxed
DNS/Git, gh authentication status and Docker socket failures were not treated as
invalid credentials/host failure or successful tests. Approved diagnostics resolved
these capability checks. Git metadata branch creation required approved execution
because .git is read-only inside the sandbox. No approval denial occurred and no
network/DNS/firewall/WSL/global setting was changed.

Sandboxed Windows cmd.exe launch failed with WSL VSOCK/interop error. The narrowly
approved version diagnostic succeeded: Windows 10.0.26300.9711. This establishes
process launch/version only, not native SDK/build/loading/GUI or Windows live DB.
Native Ubuntu/Windows CI on the actual head is separately evaluated in the report.
Local Windows Rust SDK/build was not asserted or used as proof of hosted tests.

## Local commands and observed selections

| Command / selection | Result |
| --- | --- |
| `cargo test --workspace --locked --offline` | 84 passed, zero failed/ignored/filtered; baseline preserved |
| `cargo build --workspace --locked --offline` and `--all-features` | passed |
| `cargo test --workspace --all-features --locked --offline --no-run` | passed; product CLI/compiler/full regressions ran sequentially |
| `cargo test --workspace --all-features --locked --offline -- --skip live_ --test-threads=1` | 186 passed, zero failed/ignored, ten name-substring exclusions |
| Three incidental live_ substring exclusions, individually selected | each passed: Core exact context, manifest live-claim nonpromotion, CLI live-config detecting control |
| `bash scripts/test-dispatch.sh cargo test --workspace --all-features --locked --offline -- --test-threads=1` | 196 passed, zero failed/ignored/filtered; 183 retained +13 new |
| `cargo test -p quent-cli --no-default-features --features source-review --locked --offline -- --test-threads=1` | 11 passed, zero failed/ignored/filtered |
| Same CLI command with source-review,review / source-review,manifest | 16 /15 passed, zero failed/ignored/filtered |
| `cargo fmt --all -- --check`, `git diff --check` | passed |
| `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings` | passed |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked --offline` | passed |
| `bash -x scripts/check-features.sh --offline` | 196 cargo checks passed, plus retained manifest-only/review-only CLI tests |
| All 25 documented facade output-feature combinations, test then warning-denied all-target Clippy | passed sequentially, including retained 20 combinations |
| Normal dependency trees for Core/default facade/default CLI and manifest/spec/codegen/review/source combinations | passed: Core empty, defaults Core-only, no driver/runtime for standalone offline features; existing offline features do not add source syn2 |

The final shared compiler, fresh generated consumer and product CLI build/run are
sequential. Shared compiler retains 179 negatives/nine controls and adds ten
source authority/restoration negatives and one Serialize/direct-builder fixture
compiling control: 189 checked negatives/ten controls. Negatives assert E0277 and
exclude missing-module/import failures. Fresh production-generated consumers retain
76 checked negatives/eight controls in six invocations. Two new source doctest
compile failures are separately counted among workspace tests, not folded into
shared compiler counts. Final commands do not count denied attempts or development
failures as passing evidence.

## Source observations and finite resource evidence

Independent hand-authored full-output fixtures check every stdout byte of compact
JSON and pretty JSON, final LF, empty stderr and exit3. The direct DELETE and
filter chains compile as actual Quent API examples in the positive compiler
control, while source analysis itself still makes no type-identity assertion.
The independent family/filter/state/precision expectations also cover SELECT,
COUNT, INSERT, UPDATE, leading :: and MissingWhere syntax candidates. No source
result has Complete, low, actual Blocked, execution permission or exit0.

Source input is bounded to the documented 4KiB parser envelope, 512 significant
bytes, 48 raw punctuation bytes and 16 raw grouping bytes, with fixed item/
expression/candidate/diagnostic ceilings. This is deliberately stricter than the
shared reader's 64KiB ceiling. Raw bytes in strings/comments are counted without
lexical stripping; false conservative refusals are documented. It is not a claim
of full-file Rust support or a brace-only recursive parser defense.

Tests retain unknowns for macros/derive/attributes/cfg/cfg_attr, aliases/helpers/
shadow modules/types, dynamic/control flow, unresolved receiver/other crate/same
method, mixed recognized/unknown input, nested candidates and zero candidates.
Fake normal/raw/byte/C strings and comments do not create candidates. UTF8, malformed
literal/comment/lifetime, long unary/type/generic/macro token sequences and exact
byte boundaries are covered. Real product children exercise both refused extremes
and accepted near-envelope unary/reference/generic/cast recursion. Every child
has a five-second watchdog which kills and reaps before failing; stack overflow
or abort is not accepted as a review result. No catch_unwind parser defense exists.

Public JSON/pretty/Debug/Display/error surfaces discard all canaries, parser text,
source names/values/path/span/line/snippet/token/reason/credentials. The existing
panic-hook harness tests backtrace/tracing environments; source panic/abort spellings
are data and are never executed. Tracing has an emitted detecting-event control;
TCP has a real connect/accept control and observes zero CLI connections, including
with an ambient DSN. A real file-write detecting control and child working directory
make source FS-call nonexecution detectable. No provider/DB/clock/environment API
is present in the pure module; structural zero-call evidence is distinct from
network/tracing controls and does not invent transaction observations.

Unix writerless FIFO/device/directory/final-symlink and regular/FIFO/symlink
replacement tests run for source in addition to old inputs. Both OSes have separate
source tests; Windows device/reparse/regular-reparse replacement is hosted evidence,
not inferred from WSL. Same-opened-handle and bounded bytes do not imply immutable
file snapshots, OS-wide I/O deadlines or physical server authentication.

## DB settings, cleanup and retained limits

Only new digest-pinned disposable PG16.14/MySQL8.4.8 containers were used via the
unchanged script. PG measured UTC/read committed/UTF8/en_US.utf8. MySQL measured
+00:00/REPEATABLE-READ/utf8mb4/utf8mb4_bin with ONLY_FULL_GROUP_BY,
STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,
NO_ENGINE_SUBSTITUTION; FOUND_ROWS remains pinned in the existing adapter.
Existing dedicated quent_dispatch_test/manifest_other/quent_manifest_other,
limited quent_test/hidden reader roles and generated fixtures remain isolated
inside these newly created containers. Cleanup removed dedicated containers and
anonymous volumes, then verified their absence. No pre-existing DB was consumed.

Source adds no write/Tx/live workflow; existing Strict/CRUD/validated/atomic/catalog/
CLI/codegen/patch/regen regression is preserved. All 24 fixture cases and their
runtime_pending markers remain byte-identical. Trusted capture is not implemented;
there is no self-claim JSON/source annotation path. Identity/SQL/native/model/
binding/policy/tenant/freshness/runtime remain unverified independently. SARIF,
diff/baseline/suppression are PR3. Physical server authentication, general DDL races,
TLS, Windows live DB, Windows sensitive-export/ACL success, arbitrary thread-stack
size and OS-wide I/O deadlines remain unsupported/unverified as documented.

During development one Clippy range-pattern warning and one adversary oracle
(which initially mislabeled an in-budget opaque macro as Budget) were corrected.
The macro stress now exceeds the actual documented token-byte budget. These failed
development attempts are distinct from the passing final selections above.
