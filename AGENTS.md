# Quent contributor / AI instructions

Work only within the requested Issue, planned PR, branch and base. Read the current
Issue/specification and verify required predecessors are merged into the actual
base before implementation. Preserve user changes. Goquent is reference-only;
record the commit actually read and distinguish contract comparison from runtime
or wire compatibility. Do not infer permission to merge, close Issues, start the
next work, release/tag/publish, bump versions, switch publish=true or change license.
Explicit user instructions and the active execution's permission rules take priority.

Read [README](README.md), [adoption matrix](specs/adoption-v1.md),
[integrated contracts](specs/quent-contracts-v1.md),
[all-case register](specs/integration-acceptance-v1.md) and the domain contract before
editing. Historical PR-specific absence statements/counts describe those baselines;
compare the current production API and executable oracles. Do not claim unimplemented
or unverified behavior because a specification expects it.

## Invariants

- Typed/generated/Spec and ordinary CRUD share the private immutable IR, single
  finalizer, policy/risk/final dispatch gate and finite native bind/decode path.
  Do not add alternate SQL renderers, executor callbacks or public authority restoration.
- Plan/compile/review are DB/provider0. Live catalog, EXPLAIN, connection, provider
  acquisition and migration apply are separate explicit application-only boundaries.
- Trusted independently current subject/tenant/schema/policy/settings and physical
  routing come from the application. Constructors, labels, JSON, manifest Fresh,
  Low/type success, reason, baseline and synthetic AllowCandidate are not authentication
  or permission. Unknown/Partial/unsupported must refuse, with no fallback/autoretry.
- Candidate review borrows its retained record before moving that candidate to
  acquisition; it cannot refresh TTL/reprepare/expose Inspection secrets/restore JSON.
  Consumed handles stay consumed on future creation, cancellation, error or drop.
- Preserve complete bounded result inspection, exact expectations, native count
  semantics and separate Tx attempts/ACKs. Unknown is not rollback ACK; borrowed
  completion is caller-owned. Keep same durable ID/full input for explicit command
  reconciliation; never interpret corrupt storage as missing or replace IDs automatically.
- Migration retains exact plan/phase/target/new observation, independently current
  application attestations and independent human instruction. Freshness is not apply.
- Default public output is allowlist-only: no SQL/values/names/source/path/snippet/
  free reason/opaque metadata/arbitrary error or credentials. Sensitive getters and
  detail artifacts stay local; never paste them into PR/CI/MCP. MCP correlation IDs
  reflect only at top-level /id; use innocuous IDs. Do not weaken canary controls.
- Preserve zero-dependency Core, Core-only defaults, driver/runtime-free mcp-only,
  explicit adapters/runtime/TLS, Rust1.94/edition2024/pins/lock and publish=false.
  All24 runtime_pending, original fixture expectations and loader vocabulary remain
  unchanged unless explicitly included in an independently authorized scope.

## Verification and consultation

Use [reproduction](specs/reproduction-v1.md) and the
[PR evidence template](.github/pull_request_template.md). Run checks appropriate to
the change; compile/run new executable snippets. Do not add meaningless docs mirror
tests or repeat all expensive matrices without a relevant change. Run heavy
compiler/build suites sequentially. Preserve required CI and verify its full terminal
jobset at the exact final source head. Old-head green/baseline receipts are not new
measurements; hosted Windows DB-free is not Windows local/live/ACL evidence.

For real DB tests use only scripts/test-dispatch.sh pinned disposable loopback PG/MySQL
setup, its limited fixture roles/namespaces/settings and per-run cleanup/absence.
Never use production/DATABASE_URL, prune unrelated data or change global host/DB/
DNS/firewall/WSL/Codex settings. Follow active sandbox/approval rules; a denied script
must not be rerun through Bypass/another shell/equivalent command. Missing environment,
failed tests, skipped/filtered/0selected cases and incomplete cleanup remain visible.
Preserve deterministic cancellation, independent absence/ACK and network/migration
assertions; do not relax them to obtain green results.

Consult when a decision changes scope, purpose, acceptance, authority, public data
or guarantees, or requires a new licensing/operational obligation. Explain the
concrete gap, options/recommendation and impact; do not implement a speculative
production repair inside a docs task. Decide routine reversible editing choices
within authorized scope. Report tested vs unverified separately, including blockers
and needed separate authorization. Never turn documentation preparation into a
release or commercial-safety declaration.
