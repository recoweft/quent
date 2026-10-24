Current QT-AI-05 PR2: [explicit finite catalog/CLI verification](manifest-verify-v1.md).
Earlier PR-specific absence statements below describe their historical baseline.

# QT-AI-04 planned PR2: output boundaries v1

Current implementation, superseding PR-specific historical absence statements in
older guides. Public output and local sensitive output are different contracts.
Neither is execution input, permission, a freshness proof or a recoverable handle.

## Current surface inventory

| Surface | Implementation / permitted output |
| --- | --- |
| Core sensitive IR, Ident/Value/Table/Column/Projection/Predicate, operation, record, source, metadata, context/settings/policy/schema/snapshot | Existing fixed redacted Debug; no Display/Serialize/Deserialize. Explicit `sensitive_*` borrowed getters expose data to trusted application code only |
| Plan/Inspection/PolicyPreparation public projection | PR1 allowlist PlanView, unchanged schema, JSON/Display/Debug; no source strings copied or subsequently redacted |
| Plan/Context/Policy errors | Closed enums, fixed Display/Debug, no source chain, existing DiagnosticView mapping |
| Sessions, validated/atomic handles, expectations, borrowed adapters | Existing redacted Debug; no executor/credential/context restoration from output |
| ExecutionResult / AtomicFailure / ExecutionError | Existing redacted result Debug and fixed errors; source is None. ResultView omits values/COUNT/affected counts; atomic diagnostics retain disposition |
| Serialization | Only explicit closed public `to_json()`; no production serde, general error/Display conversion, input parser or restoration interface |
| `output-tracing` facade feature | `output_tracing::emit(&impl PublicSummary)`, sealed to PlanView/DiagnosticView and feature-gated ResultView. One INFO event, literal target `quent::public`, only `public_view` field containing exact public JSON. No message, dynamic target, source Debug, arbitrary Display/Error/callback or span creation/recording |
| SQLx owned connection | Both statement and slow logging remain disabled before connect. Private per-poll/drop tracing suppression also blocks protocol notices/type-resolution fields. No global subscriber/logger is installed |
| Borrowed external SQLx connection | Caller must disable its statement/slow logs; Quent cannot reconfigure already created external connections or sanitize application subscribers/spans |
| CLI stdout/stderr | No args, help/version, unknown flag/command, review/manifest/MCP requests, malformed/huge stdin and secret paths all return exit 1, empty stdout, exactly `QUENT_NOT_IMPLEMENTED\n` stderr |
| CLI panic / output I/O | Fixed panic hook `QUENT_PANIC`, no payload/location/backtrace. stderr write errors are discarded and exit remains failure. Arguments and stdin are never read; no file open from user paths |
| Local detail | Only `local-details` feature, explicit opt-in + LocalApplication surface + absolute new path + real SensitiveSource; never a stdout/log fallback |
| MCP / SARIF / review / manifest / OperationSpec | No parser/tool implementation added. CLI remains unsupported; there is no MCP tool to exercise. Library surface denial is distinct from testing future tool transport |

No output truncation was added: the finite finding register has at most seven
entries and bounded structural counts. MissingWhere/Blocked, partial/unknown and
all complete internal facts stay intact. Actual dispatch, validation/handle
consumption, CI test verdicts and atomic expectation use complete private results,
never projected/edited views or detail export receipts. No input SQL wire, demo ORM,
manifest/review/SARIF/MCP/DB command or authority restore parser was invented.
CLI help/version are deliberately unsupported along with all other commands;
functional CLI interfaces belong to QT-AI-05/06/07/08/11/13 as applicable.

## Explicit application-local API

```rust,ignore
use quent::local_details::{create_new, LocalOptIn, Surface};
create_new(
    LocalOptIn::acknowledge_sensitive_local_file(),
    Surface::LocalApplication,
    explicitly_supplied_absolute_new_path,
    plan.sensitive_source(),
)?;
```

Inspection provides its retained `sensitive_source()`; preparation provides
`sensitive_source().final_source()`. Only that same finalized record is exported.
No context, original unprepared context snapshot, subject, session, connection,
handle, credential, permit or executor is traversed. SQL/args can themselves contain
sensitive tenant values: this file must be treated as secret. New structural
`sensitive_*` getters return borrowed names/table/projection data for the finite
serializer; Debug remains redacted. Already explicit sensitive getters are a trusted
application boundary, not a sandbox preventing arbitrary application code exfiltration.

`CreatedLocalDetail` is an opaque fixed receipt with no source/path getters,
serialization, authority conversion or execute method. `DetailError` is a closed
fixed enum/code with no retained path/OS error/source. No public bool/JSON allow,
callback serializer, environment allow key, implicit cwd filename or automatic
stdout fallback exists. The opt-in token and surface enum are application assertions,
not authentication: a malicious application can lie about its surface, remove env
or directly use sensitive getters. This contract does not authenticate process origin.

## Source artifact, output only

`kind=quent_local_source`, numeric `version=1`, separately versioned from
`quent_plan_view` / `quent_diagnostic_view` / `quent_result_view`. Top-level keys are
exactly kind/version/dialect/sql/args/metadata. Dialect is postgres/mysql. Metadata
contains exactly table/family/projection/predicate/write_data/order/limit/version/
caller_missing_where, preserving the final record's original ordered predicate and
final write/projection/version semantics (including auto policy guards). This is
not a policy/context manifest. The final SQL is copied, never reconstructed.

Table is `{path:[identifier components],alias:string|null}`; column contains table,
name,type,nullable. Projection is an ordered array of `{column,alias}`, `"count"`
or null. Predicate is null or the finite ordered and/or children, not child,
eq/ne/lt/le/gt/ge column+value, in column+values, is_null column tree. Ordered write
entries are `{column,input}`; valid finalized input is a typed value (the private
encoder explicitly distinguishes Missing). Order entries contain column and asc/desc.
Version is null or `{column,expected}`. Limit is null or a decimal string.

Every native value is `{type,value}`: type is i8/i16/i32/i64/u8/u16/u32/u64/bool/text.
Integers are canonical decimal **strings**, preserving width/sign and large u64 in
consumers with limited JSON number precision. Bool stays JSON bool, text stays UTF-8
string, NULL is JSON null WITH its native type. Empty text/false/zero/typed NULL
remain different. Quotes/backslash and every U+0000..001F are JSON escaped; the
finalizer already refuses NUL text. No `[REDACTED]` placeholder or arbitrary Debug
fallback is encoded. Payload is bounded to 16 MiB with a checked writer; over-limit
returns Budget, never truncated success. No source deserializer/restore/execute API.
This source contract is described here; it is not a public-view schema variant.

## Protection and refusal guarantees

| OS / filesystem / condition | Behavior before secret writing |
| --- | --- |
| Linux, ext4 or tmpfs, absolute path, owned non-group/other-writable destination directory | dirfd traversal of every component with O_DIRECTORY/O_NOFOLLOW; reject parent traversal and symlinks; O_WRONLY/O_CREAT/O_EXCL/O_NOFOLLOW/O_CLOEXEC final creation with mode 0600 |
| Newly created descriptor | fstat must establish effective UID owner, exact permission/special bits 0600, regular file, link count 1, empty size. Check runs before serialization/write. Restrictive umask or failed check leaves at most empty file |
| Existing file/dir/symlink/FIFO/device / concurrent create | OS exclusive-create refusal, no append/truncate/overwrite, no existence-check race. Losers never open the existing object |
| Relative/root/parent/symlink ancestor, unknown/remote filesystem, unsafe destination directory | Fixed rejection; no detail bytes written |
| Windows (including reparse paths), macOS and every non-Linux OS | UnsupportedOs before filesystem action; no ACL/owner/inheritance/access guarantee claimed |
| CI/MCP/remote product surface | SurfaceDenied before touching path |
| Known CI/remote env present | EnvironmentDenied even when value is empty/false/non-Unicode; no allow override |
| Serialization/write/fsync error | Fixed Budget/WriteFailed. No fallback, path/cause/secret log. New owner-only file can remain empty or partial; never silently delete/replace a raced pathname |

Supported Linux guarantees are conditional on authentic OS/local filesystem
semantics, a trusted destination directory, and a trusted effective owner/admin.
Other same-UID processes, privileged users, a hostile kernel/mount provider,
application-created tracing spans, encrypted-at-rest requirements and subsequent
copying/chmod/backups are outside this boundary. Mode 0600 masks group/other ACL
access; it is not Windows ACL evidence. No chmod-after-secret-write window exists.
Creation exclusivity is atomic; the payload write is not a crash-atomic publication
promise. No automatic cleanup removes a pathname after failure (could race replacement).

Known presence-denial register: CI, CONTINUOUS_INTEGRATION, GITHUB_ACTIONS, GITLAB_CI,
TF_BUILD, JENKINS_URL, JENKINS_HOME, BUILD_BUILDID, TEAMCITY_VERSION, BUILDKITE,
CIRCLECI, TRAVIS, APPVEYOR, CODEBUILD_BUILD_ID, BITBUCKET_BUILD_NUMBER, DRONE,
SEMAPHORE, WOODPECKER, QUENT_MCP, QUENT_REMOTE. Unlisted CI and a caller lying about
LocalApplication cannot be authenticated by this finite environment heuristic.
There is no MCP implementation to prove every remote transport path safe. Future
surfaces must select their denied surface and preserve this boundary.

## Dependencies and checks

Core stays dependency-free, default facade driver/runtime-free. Rust 1.94.0,
edition 2024, publish=false, SQLx=0.9.0, Tokio=1.54.1 and futures-util=0.3.34 are unchanged.
Exact tracing=0.1.44 is optional through `output-tracing` OR `runtime-tokio`: the
latter protects driver poll/drop even when public emission is disabled. Defaults are off + std, already
locked transitively; no subscriber/attributes dependency added by this feature.
`local-details` adds exact rustix=1.1.4, Linux target only, defaults off + std/fs/process:
safe wrappers for effective UID, descriptor traversal, exclusive creation, fstat and
fstatfs without weakening workspace unsafe_code=forbid. Lockfile also pins its errno
and linux-raw-sys dependencies. No normal serde, runtime, OS dependency goes into Core.
The original 16 driver/runtime/TLS subsets remain checked; independent optional
output-feature combinations are checked in addition, including native Windows refusal.

Public serialization and tracing reuse exactly the PR1 documentary schema. Existing
independent parse/key/value allowlists and full expected documents are retained;
tracing captures actual emitter target/name/fields/Debug record and independently
parses its public JSON, including real canary-derived views. A generic JSON Schema
engine was **not** used. Source JSON has separate independent exact expectations.
Native CI isolated private protection tests use only synthetic bytes/empty files;
product CI guard stays enabled and no public bypass is exported. Successful local
source exports are local evidence, not a CI export capability.

QT-C15 tests cover current surfaces, actual CLI process exits/streams, source SQL,
ordered native values/NULL/width/sign/large integers/control escaping, retained
inspection/preparation metadata, newfile/owner/mode, existing contents, symlink/dir/
FIFO/device/unsafe-parent refusal, concurrent create, actual restrictive-umask
failure with empty file, finite CI/surface denial and absence of trace fallback.
Original diagnostic/schema tests and 60 negative + 3 compiling controls are retained;
PR2 adds 11 diagnostic-checked negatives and a detail/trace compiling control.
Existing live production tests capture actual result/driver/atomic failure summaries,
and an outer subscriber observes the entire real connection/dispatch/Tx/error lifetime
to require zero unsolicited events with SQLx statement/slow logging disabled. The
outer capture excludes fixture-only raw setup/cleanup (which can emit PostgreSQL
notices); this is finite evidence on the supported ordinary-table scenarios, not
sanitization of independently executed application SQL or arbitrary server logging;
export success/denial does not change the retained gate or atomic outcome. Fixed
source errors discard external cause text rather than formatting it. No DB semantics
changed: DB-free output evidence is separate from full existing live regression.

All 24 full runtime_pending markers remain. Future Windows ACL implementation needs
native filesystem ACL/owner/inheritance/access tests, not a build-only claim. MCP,
SARIF/review/manifest/input parsing, authenticated remote origin and actual TLS/network
completion faults retain their owning issues and are not completed by output tests.
Local/hosted commands, counts, cleanup, exact head and CI results belong to the PR
and RelayWeft report. CI job presence does not establish branch protection.

## Local measurements (2026-10-24, WSL2 Ubuntu 24.04.5)

Rust 1.94.0. Default workspace: 81 tests passed including doctests. Full workspace
on the existing digest-pinned disposable PostgreSQL 16.14 / MySQL 8.4.8 fixtures:
115 passed, no failures/ignored/filtered (99 previous unique tests + 16 additions,
including the two subprocess entry tests whose actual child branches are invoked
by their parent tests). The checked API harness retains 60 negative + 3 controls
and adds 11 negative + 1 control, total 71 + 4. DB-free all-feature selection:
112 passed, three excluded by `--skip live_`; both DB suites are covered by the
full run, and the accidentally matching Core context test is covered both there
and by a separate five-test all-feature Core context run. No skip is DB evidence.

fmt, workspace default/all-feature build, warning-denied clippy/rustdoc, all original
16 feature subsets, three independent output-feature combinations and Core/default
normal dependency trees passed. Core has no dependencies; default facade has only
Core. Linux local file tests passed on the WSL EXT_SUPER_MAGIC filesystem and also
on actual tmpfs under a separately approved synthetic-only tempdir run. The Linux
filesystem magic does not distinguish ext2/3/4 or authenticate a mount provider;
local measurements do not establish every filesystem/host combination.

Full live regression retains production dispatch, finite atomic/RETURNING/version,
expectation rollback, real lock/drop/timeout/disconnect, fixed driver errors and
synthetic completion-unknown tests. Containers AND their anonymous volumes were
removed and absence verified. Dedicated DB settings/digests are unchanged from the
atomic/dispatch guides: PostgreSQL UTC/read committed/UTF8/en_US.utf8; MySQL
+00:00/REPEATABLE-READ/utf8mb4/utf8mb4_bin with the existing strict modes and FOUND_ROWS.
Real TLS handshake and actual network completion faults remain unverified.

Local Windows execution/ACLs were not attempted or verified. Native hosted Windows
CI tests product CI refusal plus the private unsupported backend, never an ACL
success claim. Hosted CI results must be checked on the actual PR head separately.


Driver suppression uses a private boxed/pinned Future with a scoped NoSubscriber
only during each poll and inner drop; no thread-local guard crosses Pending and no
application logger/subscriber is globally replaced. It wraps owned connect, validated
and atomic execution (including the shared borrowed execution adapter), independent
of opting into public tracing. It never converts driver text to a public event.
A pending/drop hostile-emission test checks suppression plus restoration for unrelated
public events, with a recorder positive control. The real PostgreSQL output-only
probe installs a disposable canary NOTICE/exception trigger: raw driver control sees
the canary, production execution exposes only fixed OutcomeUnknown/no source and the
outer subscriber sees no driver event. This deliberately false physical no-effects
assertion is solely an output test; trigger-backed DB semantics are still unsupported.

Applications remain responsible for external SQLx options parsing/pgpass/raw queries,
application spans/loggers, and downstream feature unification that explicitly enables
third-party detailed logging (in particular tracing/log-always, absent from this
workspace's pinned feature matrix). A library cannot sanitize independently installed
application loggers or prevent an application from logging its sensitive getters.
The standard tracing log-compat fallback is suppressed by the scoped dispatcher;
no global log max-level/logger configuration is changed. Known direct SQLx MySQL
cleartext-password warnings are fixed driver text, not SQL/credentials; they are not
Quent public summary events. Real TLS/certificate-diagnostic behavior remains unverified.

QT-AI-05 PR1 adds a separate optional production `manifest` source parser and
output-only `quent_manifest_summary` kind; see [manifest v1](schema-manifest-v1.md).
Earlier no-parser statements above describe QT-AI-04. Source parsing is never
public-view deserialization or authority restoration. No tracing or detail exporter
accepts the new sensitive carrier; CLI manifest remains unsupported.
