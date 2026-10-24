Current QT-AI-05 PR2: [explicit finite catalog/CLI verification](specs/manifest-verify-v1.md).
Earlier PR-specific absence statements below describe their historical baseline.

# Quent

Plan-first Rust ORM foundation for reviewing AI-generated database operations.

**Status: explicit finite schema/catalog verification (QT-AI-05 planned PR2), atop finite owned atomic execution.**
SELECT/COUNT/single INSERT/UPDATE/DELETE planning is implemented in core for PostgreSQL/MySQL.
Immediate execution uses the common policy/risk/capability gate; INSERT/High/Unknown refuse. Multi-operation transactions,
models/codegen, review, migration, commands and MCP are future work.
CLI `manifest verify --offline SOURCE COMPARISON` is DB-free; `--live SOURCE CONFIG`
explicitly runs a bounded read-only catalog observation with optional driver/runtime
features. Inputs must be explicitly named regular files; Unix FIFO inputs refuse
without waiting for a writer. Opened-handle checks and byte limits do not guarantee
OS/filesystem I/O deadlines; see the verification specification for symlink and OS
conditions. Other CLI routes retain fixed non-success refusal.
Plan/compile/inspect and validation remain DB/provider-free. Only explicit session
connect/execute and dedicated catalog-reader APIs perform I/O. No release or crates.io publication is provided.

- [Schema/manifest source, canonical fingerprints and offline evidence](specs/schema-manifest-v1.md)
- [Output surfaces, local file protections, CLI and CI/MCP limits](specs/output-boundaries-v1.md)
- [Public view schema, code register, information boundary and evidence](specs/public-views-v1.md)
- [Owned atomic results, borrowed Tx boundaries and failure/guarantee tables](specs/atomic-results-v1.md)
- [Consumed handles, exact current binding, expiry and cancellation stages](specs/owned-validation-v1.md)
- [SQLx execution, Strict gate/refusals, native matrix and real DB test procedure](specs/sqlx-dispatch-v1.md)
- [Implemented plan/type/dialect capabilities and safety boundaries](specs/plan-ir-v1.md)
- [Finite policy enforcement, support/refusal table and acceptance limits](specs/policy-preparation-v1.md)
- [Immutable settings, policy declarations and application context boundaries](specs/settings-context-v1.md)
- [Conditional scope proof and fixed structural risk rules](specs/scope-risk-v1.md)
- [Integrated contracts](specs/quent-contracts-v1.md)
- [All 24 acceptance cases and owning issues](specs/acceptance.md)
- [Architecture, dependency decisions and PR1 verification](specs/bootstrap.md)
- [Versioned fixtures and test support limits](crates/quent/tests/support/README.md)

The workspace contains `quent-core` (no dependencies), `quent` (facade with
optional SQLx dependencies), and `quent-cli` (default depends only on core; manifest/driver/runtime are optional).
Core exposes owned IR and non-executable plans with explicit sensitive inspection. LLM, runtime and database drivers
are not required for core or default workspace builds.

Rust 1.94.0 / edition 2024 is pinned. Run without a database:

```sh
cargo build --workspace --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --all-features --no-deps --locked
```

Optional adapter build checks (live behavior requires the separate test command):

```sh
cargo check -p quent --no-default-features --features postgres,runtime-tokio,tls-rustls --locked
cargo check -p quent --no-default-features --features mysql,runtime-tokio,tls-rustls --locked
```

All QT-C01–QT-C24 full runtime acceptance cases remain pending. QT-AI-00 PR2 supplied 24
language-neutral cases (69 scenarios after QT-AI-01 PR2), a bounded fixture validator, a scripted
DB call recorder and PostgreSQL/MySQL source-consumer templates. The 13 support
tests validate those tools and records; full runtime and real DB checks
remain with their owning implementation issues. Later changes add real API compile-fail tests. Expected fixture results are
contracts, never measured runtime verdicts.

QT-AI-01 PR1 adds production plan evidence for QT-C01/QT-C07 and finite core
planning tests; this does not clear full runtime acceptance or implement dispatch.
No dependencies or Cargo.lock changes are needed.

QT-AI-01 PR2 adds `Plan::inspect` over the same private finalized record. Explicit
application-asserted signed integer key facts can prove conditional maximum scope;
missing WHERE writes are structurally Blocked. Risk and scope never grant permission.
QT-C02–05/C07 have limited DB-free inspection evidence; full runtime acceptance,
live schema, execution gates and real DB results remain pending. See the scope/risk guide.

QT-AI-02 PR1 introduced private owned Strict settings, finite policy
declarations and typed target/schema/tenant/subject snapshots. DB-free preparation
inherits exact immutable content; construction does not authenticate callers or
physical databases. QT-C08/C11/C12/C15 have limited carrier/compile-fail evidence;
raw snapshots alone enforce no policy.

QT-AI-02 PR2 connects `PreparationContext::prepare_operation` to the common IR,
then the existing finalizer and inspection record. Tenant guards wrap the entire
caller predicate; explicit mismatch, tenant assignment and unsupported context
refuse. Soft-delete, PII, RequiredFilter and protected-write rules are finite and
conservative. Exact qualified canonical identities are required. Original WHERE-less
write blocks survive automatic filters. Success grants no permission. See the
[policy preparation guide](specs/policy-preparation-v1.md) for the family matrix,
limited DB-free evidence and remaining runtime/Tx/authorization responsibilities.

QT-AI-03 PR1 adds owned immediate sessions, the private shared dispatcher, finite
native binding/decoding and redacted bounded results. The app supplies authenticated
current context and authoritative database semantics; routing key equality does not
authenticate a physical DB. PR2 adds consumed delayed handles with exact current-operation/context/session checks and an exclusive monotonic deadline. PR3 adds one-operation owned atomic execution and borrowed uncommitted results. Provider permits remain future work.

Run both real DB acceptance suites against freshly created digest-pinned containers:

```sh
bash scripts/test-dispatch.sh cargo test --workspace --all-features --locked -- --test-threads=1
bash scripts/check-features.sh
# Explicit DB-free all-feature subset, also used by Windows CI:
cargo test --workspace --all-features --locked -- --skip live_
```

Live tests fail without dedicated DB opt-in; they never silently pass on missing env.
The required live-databases CI job runs them on both engines. Limited PR1 measurements
are separate from the still-pending full case responsibilities; see the dispatch guide.

Plan/Inspection/PolicyPreparation now expose finite `public_view()` copies and
allowlist-only `to_json()`/Display. Raw Plans remain unevaluated/unknown. These
outputs cannot restore execution authority. Optional `output-tracing` emits only sealed
public views and installs no global subscriber/logger. Runtime execution scopes SQLx
tracing privately during driver poll/drop, including protocol notices. Optional `local-details` provides
application-only `local_details::create_new`: explicit opt-in, local surface, absolute
new path and real SensitiveSource. Linux ext4/tmpfs owner/0600/new-file protection is
checked before secret writes; all other OS/filesystems fail closed (Windows ACL export
is unsupported). CI/MCP/remote refuse; environment detection does not authenticate an
application. Files are output-only and cannot restore execution authority. See the
output guide for exact conditional guarantees and CLI non-success behavior.

Optional `manifest` adds a strict bounded sensitive JSON source parser and independent
schema/policy/generated-code/live-claim canonical fingerprints. Offline exact-content
comparison never grants live freshness or execution authority. Public summaries are
output-only; Core and the default facade retain their dependency boundaries.

Live observation is private in-process evidence with exact target/context/connection
binding and exclusive monotonic expiry; it grants no execution authority. PG/MySQL
default ambiguity, metadata privileges, finite support and snapshot/DDL conditions
are explicit in the [verification matrix](specs/manifest-verify-v1.md).
