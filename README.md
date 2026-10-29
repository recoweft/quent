# Quent

Plan-first Rust ORM for reviewing database operations before explicit application
execution. The current v0.x workspace supports **finite** PostgreSQL/MySQL
operations; it is not a general SQL executor or an autonomous database agent.

Rust **1.94.0**, edition **2024**, is pinned in [rust-toolchain.toml](rust-toolchain.toml).
Use this repository checkout: publication, crate-name availability and a released
installation path are unverified. Workspace `publish = false` remains in place.

## Start without a database

From the repository root, with the pinned toolchain and locked dependencies cached
(or registry access for their first download):

```sh
cargo build --workspace --locked
cargo test --workspace --locked
cargo run -p quent --example tenant_review --no-default-features --features codegen,review --locked
```

The [executable example](crates/quent/examples/tenant_review.rs) generates and compares
exact Rust artifacts, binds nominal model keys and projections, inspects three-state
patches, reviews offline manifest domains and plans a separate finite nullable-column
migration. It exits 0 after its assertions and prints six public JSON lines. It
uses **synthetic application assertions**, supplies no authentication or execution
permission, and performs no DB/provider calls. Patches are inspection-only.
See [setup, outputs, refusals and reproduction](specs/reproduction-v1.md).

`quent-core` has zero normal dependencies. Default `quent` and `quent-cli` depend
only on Core; neither selects a driver/runtime. `mcp` alone also selects neither.
Application runtime builds explicitly choose an adapter, runtime and TLS:

```sh
cargo check -p quent --no-default-features --features postgres,runtime-tokio,tls-rustls --locked
cargo check -p quent --no-default-features --features mysql,runtime-tokio,tls-rustls --locked
```

These are build checks, not live connections or proof of TLS interoperability.
[Feature definitions](crates/quent/Cargo.toml) and
[CLI feature definitions](crates/quent-cli/Cargo.toml) are authoritative.

## Application execution and refusal boundaries

Builders, typed Spec and generated reads feed the common private immutable IR,
finalizer, policy/risk gate and native dispatcher. Plan/compile/review do not call a
DB or authorization provider. Public JSON, type success, reason, manifest Fresh
and a Low classification never authenticate a user or grant permission.
The application supplies trusted **independently current** subject, tenant, schema,
policy, settings and physical routing; constructors do not authenticate them.

For the existing finite authorized UPDATE/physical DELETE flow, create a candidate,
call **that candidate's `review()`**, then move the same candidate into
`acquire_authorized`. Acquisition explicitly invokes the registered trusted adapter;
`execute_authorized_atomic` consumes the permit and rechecks the current request,
context, expiry and operation at the final common gate before native submission.
Repeated review is DB/provider0, cannot extend TTL, does not reprepare and returns
no Inspection secret getter or executable JSON. The complete compiled/live sequence
and negative controls are in
[the tenant acceptance source](crates/quent/src/execution/tests/acceptance.rs) and
[authorization contract](specs/authorization-gate-v1.md).

| Finite positive path | Required refusal / limit |
| --- | --- |
| Complete supported ordinary SELECT/COUNT/Low keyed UPDATE or physical DELETE | Original WHERE-less writes remain Blocked; ordinary INSERT, High and Unknown refuse |
| Explicit finite High UPDATE/physical DELETE authorization | No permit can waive tenant mismatch, structural block, incomplete/native/schema refusal |
| Owned exact-expectation execution, full bounded result consumption and completion ACK | Lost ACK/cancel is Unknown; borrowed results remain caller-owned Uncommitted |
| Durable command exact1 and fixed business/audit/outbox bundle | Same-ID different input conflicts; corrupt ledger refuses; external delivery is not exactly-once |
| Offline review, manifest comparison, generated model/key/patch | No public view/Fresh/type success → permit/apply; no patch execution |

Read the [adoption and Go migration guide / support-refusal matrix](specs/adoption-v1.md)
before selecting an execution path. SQL, values, identifiers, source, arbitrary errors
and credentials belong outside default public output. Sensitive application getters
must not be logged. Synthetic `AllowCandidate` controls are not a production
identity provider or evidence of production least privilege.

## Read-only tools and remaining work

[Offline CLI and finite MCP stdio](specs/reproduction-v1.md#offline-cli-and-mcp)
provide bounded public analysis. MCP implements a finite 2025-11-25 profile: the
ordinary CLI has no typed model registration and refuses plan/compile with
`missing_application_context`. There are no MCP CRUD/Raw/apply/permit/live/export
tools. Application Rust registration and successful fixture children are distinct.

[All 24 full acceptance cases remain `runtime_pending`](specs/integration-acceptance-v1.md).
Finite passing subcases and original fixture expectations do not clear those markers.
[Go comparison](specs/goquent-case-comparison-v1.md) is a pinned documentary contract
comparison; Go runtime and wire compatibility remain unverified.

- [Current adoption, limits and source-linked feature matrix](specs/adoption-v1.md)
- [Reproduction, exact counts and CI interpretation](specs/reproduction-v1.md)
- [Publication checklist: prepared, unresolved, separately authorized](specs/release-checklist-v1.md)
- [Integrated specification baseline](specs/quent-contracts-v1.md) and [case responsibility history](specs/acceptance.md)
- [AI contributor instructions](AGENTS.md) and [PR evidence template](.github/pull_request_template.md)

Older PR-specific absence statements in specifications describe their named baseline;
use the current adoption matrix and linked later contracts for implemented extensions.
Documentation preparation does not announce release, commercial adoption safety or
completion of full runtime acceptance. Release/tag/publish, a version bump,
`publish = true` and license changes require separate explicit instructions.

SQLite adoption planning: [bounded contracts and independent anonymous fixture](specs/sqlite-adoption-v1.md).
SQLite remains unimplemented; these future expectations do not clear runtime_pending.
