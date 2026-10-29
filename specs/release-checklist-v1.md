# Publication preparation and open checks

QT-AI-14 PR2 prepares documentation for the current finite v0.x workspace. This is
an evidence checklist, not release authorization, new licensing terms, an operating
service agreement or a commercial safety certification. Documentation edits alone
cannot clear pending acceptance or unsupported features.

## Prepared in source (verify against the PR's exact head)

- [README](../README.md) starts with the DB-free executable example and explicit
  feature/runtime boundaries, rather than historical planned feature claims.
- [Adoption matrix](adoption-v1.md) links supported/refused paths to current contracts,
  production source and existing positive/negative oracles, including Go differences.
- [Reproduction guide](reproduction-v1.md) separates commands, non-success analysis,
  dependency/fixture setup, cleanup, counters, baseline and new-head measurements.
- [AI instructions](../AGENTS.md) preserve the common final gate, trusted current
  application context, confidentiality, unknown refusal and consultation conditions.
- [PR template](../.github/pull_request_template.md) captures source/Issue/planned PR,
  scope, case/counter/OS/CI evidence, failures/skips and remaining limits.
- Workspace version0.1.0, edition2024/Rust1.94, pins/lock, MIT license and
  `publish = false` are retained. These are source facts, not publication checks.

## Verification required for the proposed source

- [ ] Check documentation commands and feature/CLI grammar against implementation.
  Compile/run new executable snippets; identify conceptual fragments explicitly.
  Never claim `ignore` examples were tested as complete applications.
- [ ] Record actual example output/exit, default and needed all-feature/compiler/
  generated-consumer regressions; do not replace them with PR51 receipts.
- [ ] Complete fmt/build/warning-denied Clippy/rustdoc and relevant dependency trees.
- [ ] Preserve all94 expected CI jobs and confirm each terminal conclusion for the
  final source head, including hosted Ubuntu/Windows and pinned both-DB dispatch.
- [ ] For each actual DB run, record setup versions/settings, business vs auxiliary/
  provider/Tx attempts+ACK/observer counters, failures and owned cleanup/absence.
- [ ] Check PR/report/public artifacts for source/details/SQL/value/canary/arbitrary
  errors/DSNs/credentials. Redaction must not weaken private assertions.

Checked results for PR2 are in [evidence](adoption-evidence-v1.md) and its separate
exact-head PR/RelayWeft report. Empty boxes here define review checks; they are not
a claim that the checks failed or a mechanism to self-approve publication.

## Retained blockers / unsupported / unverified

| Item | Current state / decision needed |
| --- | --- |
| All QT-C01–QT-C24 original full runtime acceptance | All24 runtime_pending remain unchanged. Finite subcases are documented separately; docs do not mark them complete |
| Broader ORM capability | General batch/upsert/join/Raw, richer native types, patch execution, arbitrary multi-op Tx remain unsupported; decide separately if adoption requires them |
| External trust | Current context/subject/tenant/schema/policy, physical routing, accurate no-effects declarations and provider authentication remain application responsibilities, not verified identity products |
| Durable operations | Ledger installation/retention/privilege integrity and explicit same-ID reconciliation remain application responsibilities; no external delivery exactly-once |
| Migration | Finite nullable signed-integer tail ADD only; partial observation + independent current attestations/human instruction; no universal rollback or automatic recovery |
| Performance/observation | Non-ANALYZE Partial estimates, supported-path non-Tx attempt budgets, separate Tx ACKs and N+1 suspicion only; no measured improvement/general monitoring promise |
| MCP | Finite read-only stdio profile; ordinary CLI typed registration unavailable; no full-client/SDK/wire compatibility or hard OS deadline guarantee |
| Windows local | Prior dependency/unsigned-script policy stop was before tests. Local process/SDK, live DB and ACL evidence remain unverified/unsupported as documented; hosted DB-free does not clear them |
| Network/DB semantics | General TLS/transport failures, arbitrary server versions/extensions/schema effects/concurrent DDL and external exclusion services are not established by fixture success |
| Distribution | Crate/product-name availability, released installation URLs, packaging/publication and consumer compatibility beyond checked finite fixtures remain unverified |
| Go comparison | Pinned contract table only; no remote-current/runtime/wire compatibility claim |

If one of these is required for a release/adoption decision, report the precise unmet
check and request a separate scope/acceptance decision. Do not implement a repair,
remove a runtime_pending marker, reduce required CI or invent a guarantee in this PR.
No new license, support/retention SLA, release threshold or operational obligation is
inferred here; the linked existing contracts retain their conditional assumptions.

## Separate explicit authorization

Release creation, tags, crates.io publish, `publish = true`, version bump and license
changes are outside this PR. Merge, Issue closure and starting another work are also
outside worker authority. Documentation or green CI does not authorize any of them.
Before any later publication request, its owner must explicitly decide the scope,
remaining unmet checks and distribution actions; this checklist declares none complete.
