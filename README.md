# Quent

Plan-first Rust ORM foundation for reviewing AI-generated database operations.

**Status: bootstrap and contract test support (QT-AI-00 PR1/PR2). No ORM runtime is implemented.**
CRUD, SQL planning/validation, tenant policy, execution gates, transactions,
models/codegen, manifests, review, migration, commands and MCP are future work.
The CLI currently exits unsuccessfully with the fixed code `QUENT_NOT_IMPLEMENTED`.
No database is contacted. No release or crates.io publication is provided.

- [Integrated contracts](specs/quent-contracts-v1.md)
- [All 24 acceptance cases and owning issues](specs/acceptance.md)
- [Architecture, dependency decisions and PR1 verification](specs/bootstrap.md)
- [Versioned fixtures and test support limits](crates/quent/tests/support/README.md)

The workspace contains `quent-core` (no dependencies), `quent` (facade with
optional SQLx dependencies), and `quent-cli` (depends only on core).
The libraries currently expose no ORM API. LLM, runtime and database drivers
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
language-neutral cases (66 scenarios), a bounded fixture validator, a scripted
DB call recorder and PostgreSQL/MySQL source-consumer templates. The 13 support
tests validate those tools and records; runtime, real DB and compile-fail checks
remain with their owning implementation issues. Expected fixture results are
contracts, never measured runtime verdicts.
