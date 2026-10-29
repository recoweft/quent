# Current v0.x adoption and migration guide

This is the QT-AI-14 PR2 entry into the current finite contracts, following merged
PR51. It does not broaden production capabilities. The integrated specification
is [quent-contracts-v1](quent-contracts-v1.md); original expectations and all 24
runtime_pending markers remain in [acceptance](acceptance.md). Use the
[current case-to-source register](integration-acceptance-v1.md) to distinguish
finite evidence from unmet full expectations. Historical PR counts/absence statements
are evidence of those PRs, not a fresh measurement or the current feature matrix.

## Choose a boundary before adopting

Start with the DB-free [tenant_review example](../crates/quent/examples/tenant_review.rs)
and [reproduction guide](reproduction-v1.md). Core/default builds need no DB/runtime;
`mcp`, `codegen`, `manifest`, `operation-spec` and review features alone introduce
no driver/runtime. Enable `postgres` or `mysql`, `runtime-tokio`, and your explicit
TLS selection for an application connection. Locked SQLx 0.9.0/Tokio 1.54.1 and Rust
1.94.0/edition2024 are repository pins, not a claim about other SDK/product versions.
See [facade](../crates/quent/Cargo.toml), [CLI](../crates/quent-cli/Cargo.toml),
[Core](../crates/quent-core/Cargo.toml) and [workspace](../Cargo.toml).
No published install command or crates.io name availability has been verified.

The application must authenticate its subject and tenant, supply accurate complete
schema/policy/settings, independently current snapshots and correct physical routing,
and register any trusted authorization adapter. Labels and constructors do not do
this work. Replaying an obsolete snapshot cannot establish currentness. Supported
ordinary-table/no-effects assertions do not verify hidden triggers/RLS/extensions,
concurrent DDL, a physical server or an external identity service.

Default public output carries fixed codes/enums and bounded structure, not SQL,
values, names, tenant, source paths/snippets, free reasons, opaque metadata or
arbitrary errors. Explicit sensitive getters are application-local and must not be
copied into PRs/logs/CI/MCP. Protected local detail files are opt-in new-file output,
Linux-only under the documented filesystem/owner checks; Windows ACL export is
unsupported. See [output boundaries](output-boundaries-v1.md) and
[public views](public-views-v1.md). Never check in a DSN/credential or use a real
secret as a correlation ID.

## Capability and refusal matrix

“Supported” below means the linked **finite** domain under its application
assumptions, not completion of the original case. Production sources and independent
live/compiler oracles are linked so an adoption decision can check the actual API.

| Domain / features | Current finite path | Refusal / unverified boundary | Contract and implementation / oracle |
| --- | --- | --- | --- |
| Core planning / default | Explicit SELECT, COUNT, single INSERT, UPDATE, physical DELETE; immutable final SQL/ordered native args/metadata | Plan is non-executable; Raw/join/CTE/subquery/upsert/general batch absent | [IR](plan-ir-v1.md), [Core source](../crates/quent-core/src/lib.rs) |
| Policy / scope | Full caller predicate wrapped by tenant AND; original missing-WHERE retained; signed nonnull key/tenant proof | Missing/mismatched tenant, tenant assignment, protected write, unknown effects/coverage refuse; id name/LIMIT is no key proof; soft-delete can remain Partial | [policy](policy-preparation-v1.md), [scope](scope-risk-v1.md), [Core tests](../crates/quent-core/src) |
| Ordinary runtime / driver + runtime | Complete permitted SELECT/COUNT/Low Zero-or-Single UPDATE/DELETE through shared final gate | Ordinary INSERT/High/Unknown refuse; explicit connection is I/O; public plans do not dispatch | [dispatcher](sqlx-dispatch-v1.md), [execution](../crates/quent/src/execution.rs), [tests](../crates/quent/src/execution/tests.rs) |
| Native values | PG i16/i32/i64/bool/text; MySQL finite signed/unsigned widths/bool/text; exact nullable native NULL | Decimal/UUID/bytes/date/time/float unsupported; no coercion; proof domain narrower than value domain | [native matrix](typed-operation-spec-v1.md), [PG](../crates/quent/src/execution/postgres.rs), [MySQL](../crates/quent/src/execution/mysql.rs) |
| Typed OperationSpec / operation-spec | Single-model read, explicit fields/order/limit, full application mapping; owned compiled read with driver/runtime | Wire tenant predicates/overrides, writes/Raw/joins/aggregates refuse; compilation may succeed yet execution refuse Partial/High | [typed Spec](typed-operation-spec-v1.md), [source grammar](operation-spec-v1.md), [compiler](../crates/quent/src/operation_spec/typed.rs), [oracles](../crates/quent/tests/typed_spec.rs) |
| Generated model/key/read / codegen | Explicit v1/v2 pure generation, nominal model/key/predicate/order/projection rows; independent current mapping bind; owned read | No universal derive, implicit projection, arbitrary native type, generated borrowed/atomic write repository or generator CLI | [generation](typed-codegen-v1.md), [emitter](../crates/quent/src/codegen/emit.rs), [consumer tests](../crates/quent/tests) |
| Three-state patch / codegen v2 | Unchanged vs SetNull vs Set(T), model-specific consumed patch; common preparation/inspection | Inspection-only, no execute conversion; protected/tenant setters absent; duplicate/empty patch refuses | [patch/regen](typed-patches-regeneration-v2.md), [patch source](../crates/quent/src/codegen/patch.rs), [example](../crates/quent/examples/tenant_review.rs) |
| Regeneration / codegen | Exact previous/proposed/existing bytes and domains; NoChange/change/conflict | No filesystem overwrite/apply; CRLF not normalized; no v1→v2 silent upgrade; generated data never trusted context | [contract](typed-patches-regeneration-v2.md), [comparison](../crates/quent/src/codegen/regenerate.rs) |
| Delayed / owned atomic / borrowed | Consumed exact input/context/session/TTL binding; owned one-operation expectation 0..256, finite version and PG UPDATE/DELETE RETURNING | No arbitrary multi-op Tx/savepoint/callback; MySQL RETURNING refuses; borrowed completion belongs to caller | [handles](owned-validation-v1.md), [results](atomic-results-v1.md), [runtime](../crates/quent/src/execution.rs) |
| Authorization / driver + runtime (+review for candidate review) | Explicit finite High UPDATE or physical DELETE candidate → same-record review → request/session-bound acquisition → final dispatch | Ordinary High still refuses; unbound receipt not executable; adapter cannot waive Blocked/Unknown/native/tenant/PII refusal, INSERT/version/RETURNING/batch unsupported here | [final permits](authorization-gate-v1.md), [source](../crates/quent/src/authorization.rs), [tenant oracle](../crates/quent/src/execution/tests/acceptance.rs) |
| Durable command / driver + runtime | All-column single INSERT (only absent tenant supplemented), Complete Low UPDATE/version, exact1; installed ledger and explicit lookup | No High/borrowed/RETURNING/general batch; malformed/pending/version-invalid ledger refuses; no missing→automatic write | [ledger](command-ledger-v1.md), [runtime](../crates/quent/src/execution/command.rs), [network oracle](../crates/quent/src/execution/tests/command_network.rs) |
| Audit/outbox bundle / driver + runtime | Exactly business1 + audit INSERT1 + outbox INSERT1 in same owned Tx and ledger; explicit tenant role mapping | No variable Tx/callback/consumer/delivery worker; external effects not exactly-once; plain↔bundle same key conflicts | [bundle](command-bundle-v1.md), [oracle](../crates/quent/src/execution/tests/command/bundle.rs) |
| Manifest / manifest | Schema, policy, generated-code and live-claim domains independent; bounded source parsing/offline comparison | Equality/offline consistency/public Fresh cannot construct trusted schema, permit or apply; claimed live stays unverified | [source](schema-manifest-v1.md), [parser](../crates/quent/src/manifest.rs) |
| Catalog / manifest + driver + runtime | Explicit dedicated finite read-only catalog observation and exact context/session/scope/time comparison on PG16.14/MySQL8.4.8 | Missing/normalized default/visibility can be Unknown; MySQL DDL consistency requires external exclusion; no full catalog/effects proof | [verification](manifest-verify-v1.md), [readers](../crates/quent/src/manifest/live) |
| Migration plan / Core | Separate typed before/after declarations and private plan/phase/range; finite comparison | No ManifestSource/Fresh→MigrationSchema; unknown differences refuse all SQL | [plan](migration-plan-v1.md), [source](../crates/quent-core/src/migration.rs) |
| Migration preflight / manifest + driver + runtime | Exact plan/Expand/target/role/visibility/exclusion, new finite observation | Partial/full-effects Unverified, no apply conversion; Backfill/Contract not general work | [preflight](migration-preflight-v1.md), [source](../crates/quent/src/manifest/live/preflight.rs) |
| Human migration apply / same features | Dedicated apply connection, new observation, five independent current application attestations and separate human instruction | Only constraint-free nullable signed-integer tail ADD; no automatic recovery/continuation; PG ACK Tx differs from MySQL implicit prefix/Unknown | [apply](migration-apply-v1.md), [source](../crates/quent/src/manifest/live/apply.rs) |
| EXPLAIN / performance-plan + driver + runtime | Explicit SELECT-only non-ANALYZE acquisition, existing record/native binding, closed server JSON subset | Partial/Estimated, not measurements/improvement/permission; COUNT/DML/unknown operator/version refuse; no CLI/MCP acquisition | [performance](performance-plan-v1.md), [source](../crates/quent/src/execution/performance.rs) |
| Request observation/budget / driver + runtime | Ordinary/delayed/owned-atomic/borrowed and owned compiled-read scopes; supported non-Tx SQL attempt ceiling | Authorized/command/generated wrapper/EXPLAIN/catalog/migration not covered; no refund; Tx separately observed; N+1 is suspicion | [observation](query-observation-v1.md), [source](../crates/quent/src/execution/observation.rs) |
| Offline review / review, source-review, review-ci | Fixed public plan/spec/manifest analysis; opt-in syntax candidates; supplied-input diff/SARIF/expiry | No whole-program/type/runtime proof, source coverage never Complete; baseline not a waiver; review has no exit0 | [review](review-v1.md), [source](source-review-v1.md), [CI](review-ci-v1.md), [CLI](../crates/quent-cli/src/main.rs) |
| MCP / mcp only is driver/runtime-free | Finite stdio read-only 2025-11-25; four tools/four resources; optional explicit application Rust typed registration | Ordinary CLI missing_application_context; no CRUD/Raw/apply/permit/live/export; no all-client/hard OS deadline guarantee | [protocol](mcp-v1.md), [server](../crates/quent/src/mcp.rs), [transport](../crates/quent/src/mcp/transport.rs), [child tests](../crates/quent/tests/mcp_stdio.rs) |

## Review the same candidate, then acquire explicitly

The real callable sequence is in
[acceptance.rs](../crates/quent/src/execution/tests/acceptance.rs), compiled under
`review` plus driver/runtime and exercised by the dedicated both-DB suite. It
constructs an operation and `authorization_candidate`, calls its `review()` repeatedly
before acquisition, moves that same candidate into `acquire_authorized`, then
consumes the permit in `execute_authorized_atomic`. No illustrative `ignore` snippet
is offered as a standalone runnable application.

Review is the existing public Review projection over the retained private preparation;
it neither rerenders/reprepares nor invokes DB/provider, exposes Inspection secrets,
refreshes TTL or restores from JSON. A Refused review of an ordinary High operation
is not overridden by its text; only the eligible explicit acquisition path may
produce the bound permit. The provider control's `AllowCandidate` response is a
test response, not authentication or a production least-privilege product.

Ordinary High refusal, changed independently current subject/request/session/content,
expiry after review, provider Deny/Error/Cancelled, and operation substitution remain
zero-business-send counterexamples. The final gate/native bind uses the retained
record. Matching public JSON alone is not correspondence proof; the oracle also
compares retained SQL allocation, ordered arguments and metadata.

## Completion and reconciliation

Owned atomic execution checks all bounded results/columns and the expected count
before commit. PG RETURNING consumes the complete set; a first row is insufficient.
MySQL's pinned FOUND_ROWS reports matched rows, including an unchanged match; it
must not be interpreted as changed-row count. Version conflict and count/decode
failure do not commit; only rollback ACK is RolledBack. Commit/rollback loss,
post-send cancellation or dropped future can remain Unknown, with session quarantine.
Drop's best-effort cleanup and a quiet observer are not completion ACKs.
Borrowed execution is Uncommitted and the application owns commit/rollback; an error
cannot enforce rollback-only or promise atomic expectations.

For commands, persist the original full input, durable ID, namespace and authoritative
ledger placement before the first send. Reconciliation explicitly validates the
same ID/input/current authority and uses lookup in a fresh session. Same-ID exact
input may replay saved results without business sends; different input conflicts;
malformed/unknown-version/pending ledger refuses. A successful missing lookup is
not evidence another transaction has finished. Do not replace IDs, treat invalid
storage as absent or automatically re-execute. Installation/retention/privilege
integrity remains application responsibility. DB audit/outbox rows share the owned
transaction; external delivery has no exactly-once guarantee.

Migration needs its separate original typed plan, exact phase/target/preflight,
new current application attestations and independent human instruction. Manifest
Fresh, type success and public JSON cannot apply it. PostgreSQL COMMIT/ROLLBACK ACK
outcomes differ from MySQL's persistent acknowledged ALTER prefix, implicit commit
and unknown step; neither means universal rollback or automated recovery. Even the
finite nullable tail ADD can block workloads.

## Moving from Goquent

The [case comparison](goquent-case-comparison-v1.md) uses the actually read local
commit `c51e084d0d530cae3eb22ad2b0686e47c6b79f0c` contract table. It establishes no
remote-latest, Go runtime, wire/SDK or SQL-byte compatibility. Goquent is reference-only.

1. Inventory each used operation against the matrix, native types and full case
   expectations. Arbitrary SQL, compatibility reason overrides, joins/upserts,
   generic batch and decimal/UUID/time/bytes need separate decisions; this PR adds
   no fallback or repair for them.
2. Supply trusted server current context and complete typed mapping explicitly.
   Do not port user JSON tenant, mutable plans, fresh claims or reason strings as
   authority. Default Strict has no legacy low-risk receiver fallback.
3. Replace structural key conventions with model-specific nominal keys and explicit
   projections. Preserve native widths, Option and Unchanged/SetNull/Set distinctions.
   Generation is pure local code data; exact regeneration comparison does not
   overwrite files, authenticate schema or turn a patch into a write API.
4. Select owned single-operation completion or caller-owned borrowed semantics.
   Separate native matched counts, complete result inspection and Tx ACKs; do not
   port autocommit count errors as rollback guarantees.
5. For the finite durable command subset, install the explicit dialect ledger and
   retain its namespace/ID/full input. Reconcile Unknown explicitly. Audit/outbox
   database atomicity does not port external callback exactly-once assumptions.
6. Reproduce supported positives and mandatory refusals with dedicated fixture
   databases before making an adoption decision. Record unverified OS/live/TLS,
   schema and operational assumptions via the [PR template](../.github/pull_request_template.md).

No row is a commercial safety certification. Full fixture acceptance, local Windows
execution/live/ACL, arbitrary TLS/network faults, external provider authentication,
physical routing and broader schema/effects remain separate unverified or unsupported
items. The [publication checklist](release-checklist-v1.md) preserves these holds.
