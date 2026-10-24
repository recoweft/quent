# Quent

Plan-first Rust ORM foundation for reviewing AI-generated database operations.

**Status: DB-free plans/inspection plus immutable per-database application snapshots (QT-AI-02 PR1). No execution runtime is implemented.**
SELECT/COUNT/single INSERT/UPDATE/DELETE planning is implemented in core for PostgreSQL/MySQL.
Tenant policy, execution gates, transactions,
models/codegen, manifests, review, migration, commands and MCP are future work.
The CLI currently exits unsuccessfully with the fixed code `QUENT_NOT_IMPLEMENTED`.
No database is contacted. No release or crates.io publication is provided.

- [Implemented plan/type/dialect capabilities and safety boundaries](specs/plan-ir-v1.md)
- [Immutable settings, policy declarations and application context boundaries](specs/settings-context-v1.md)
- [Conditional scope proof and fixed structural risk rules](specs/scope-risk-v1.md)
- [Integrated contracts](specs/quent-contracts-v1.md)
- [All 24 acceptance cases and owning issues](specs/acceptance.md)
- [Architecture, dependency decisions and PR1 verification](specs/bootstrap.md)
- [Versioned fixtures and test support limits](crates/quent/tests/support/README.md)

The workspace contains `quent-core` (no dependencies), `quent` (facade with
optional SQLx dependencies), and `quent-cli` (depends only on core).
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

Optional dependency compatibility checks (these do not implement adapters):

```sh
cargo check -p quent --no-default-features --features postgres,runtime-tokio,tls-rustls --locked
cargo check -p quent --no-default-features --features mysql,runtime-tokio,tls-rustls --locked
```

All QT-C01–QT-C24 runtime acceptance cases remain unverified. PR2 supplies 24
language-neutral cases (69 scenarios after QT-AI-01 PR2), a bounded fixture validator, a scripted
DB call recorder and PostgreSQL/MySQL source-consumer templates. The 13 support
tests validate those tools and records; runtime, real DB and compile-fail checks
remain with their owning implementation issues. Expected fixture results are
contracts, never measured runtime verdicts.

QT-AI-01 PR1 adds production plan evidence for QT-C01/QT-C07 and finite core
planning tests; this does not clear full runtime acceptance or implement dispatch.
No dependencies or Cargo.lock changes are needed.

QT-AI-01 PR2 adds `Plan::inspect` over the same private finalized record. Explicit
application-asserted signed integer key facts can prove conditional maximum scope;
missing WHERE writes are structurally Blocked. Risk and scope never grant permission.
QT-C02–05/C07 have limited DB-free inspection evidence; full runtime acceptance,
live schema, tenant gates and real DB results remain pending. See the scope/risk guide.

QT-AI-02 PR1 adds private owned Strict settings, finite **unenforced** policy
declarations and typed target/schema/tenant/subject snapshots. DB-free preparation
inherits exact immutable content; construction does not authenticate callers or
physical databases. QT-C08/C11/C12/C15 have limited carrier/compile-fail evidence;
full policy, dispatch and Tx acceptance remains pending. See the snapshot guide.
