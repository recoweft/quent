# QT-AI-11 PR1 finite evidence

Work bffca4b2-17a6-4c5c-acd5-e030451d776e revision 1; public execution
`db9c9f38-f2a5-4d88-803b-969a933c29dd`. This is finite PR1 evidence, not Issue12
completion, preflight/application acceptance, universal DDL support, or clearance
of any of the 24 full runtime_pending fixture markers.

## Identity and prerequisites actually checked

Actual cwd `/home/murai/github/quent`, clean starting tree, no applicable AGENTS.md
in the repository/ancestor paths; `.agents`/`.codex` contain no instructions.
Ubuntu 24.04.5 on Microsoft WSL2; rustc 1.94.0 matches rust-toolchain.toml.
SSH origin fetch/push is `git@github.com:recoweft/quent.git`. GitHub worker API
identity `recoweft`, repository push/admin permissions, specified-branch push dry-run
and PR API reads were checked independently from Issue connector authentication.
Read-only remote main was bb268ce92d7c7c7920825af34175b718c7261bab. After fetch,
PR26 and PR28 were confirmed merged/base main with respective merge commits
4d3c7e3832543af8e9c72f30814c5316e938db71 and
26377bdada75d5a57a6a87ad35cf9d4ec28bbdba, both ancestors of actual origin/main.
The requested branch was created from that fetched main, preserving the clean
prior PR41 checkout. Issue1/12 were read through authenticated worker API.

Sandbox DNS failure and Docker socket denial did not establish invalid credentials
or a broken host. Narrow approved remote/auth/Docker diagnostics succeeded
(Docker client/server 29.8.0); approved fetch/branch creation and disposable DB
regression subsequently succeeded. Initial sandbox git switch failed because
`.git` is read-only; approved branch creation succeeded. No network, DNS, WSL,
firewall, global Codex or application setting was changed. These facts do not
verify arbitrary Windows programs/SDK/GUI, general network faults or TLS.

## Cases, APIs and guarantees

| Case | Production/test boundary | Actual finite result |
| --- | --- | --- |
| C22 | Core typed `MigrationSchema`, `plan_migration`, private `MigrationPlan` | both dialects: NoChange known finite input, supported nullable append, changes/refusals for columns/type/null/default/index/FK/CHECK; namespace/target/table/common-column/component/enumeration order and same-name definition differences |
| C22/C19 | Knowledge, Observed, TableEffects, opaque defaults/indexes/CHECKs | equal unknowns remain Refused; all domain coverage enumerated; supported SQL mixed with unsupported or final-table unknown leaves zero SQL |
| C01/C12 | exact owned snapshots/repeated public view and SQL | 32 repetitions/dialect deterministic; caller snapshot mutation after planning does not change retained plan |
| C08/C19 | authority/target fields, manifest compatibility, no conversions | declarations stay Unverified live; changed full target/authority refuses; old manifest extension fields and migration view input refused |
| C15 | redacted sensitive Debug, fixed errors/no source, independent public JSON allowlist, local exporter | actual sensitive SQL contains detecting canaries; default public strings omit names/types/SQL/default expression/values; full existing CLI/JSON/SARIF/tracing regression retained |
| C15/C23 | `create_new_migration`, common protected local backend | explicit new file 0600/link count1, exact retained SQL and difference names, no overwrite; CI/MCP/Remote denied, shared environment-child denial checks both exporters; native non-Linux backend test requires UnsupportedOs/no file |
| C12/C15/C23 | real compiler harness | plan/view/schema cannot deserialize/serialize or become context/facts/validated authority; private SQL inaccessible; no apply method; CRUD validate rejects migration plan; matching positive control compiles |
| C24 | bounded typed constructors | exact 4096/4097 opaque/text, 128/129 target, 16/17 components/objects/tables, 256/257 columns, duplicates/dangling/type mismatch/case collisions; 4080 maximum supported steps/dialect and final unknown clears all |

There are 15 new Core runtime tests, two new Core compile-fail doctests, three new
facade tests (one default, one manifest, one local-details) and two test-only live
suites. New cases are assertions over the production APIs, not new production
execution pathways. The independent fixture recorder has DB0 and no Tx events
around pure planning; provider0 follows from Core's empty dependency tree and no
provider/executor/callback input or call site. This is not a simulated live
rollback/refusal measurement. Standalone provider acquisition is not invoked.
Full source parser malformed/duplicate/size/depth/unknown-version/multi-document
regressions remain in existing tests. No migration input wire exists; every JSON
restoration is rejected by the absence of the requisite API/trait. A separate
output-only schema is checked by independent emitted-key/value expectations,
not by a general JSON Schema engine.

## Test-only SQL and independent observations

`migration_live.rs` is compiled only with both drivers and runtime. It uses raw
SQLx solely in tests to create an isolated table, apply the exact finalized SQL,
observe information_schema and rows, and drop the table. Its AssertSqlSafe wrapper
is test-only and accepts only the planner-generated finite SQL; there is no such
migration path in product code. Product plan/review/compile dispatch count remains
0 and provider count 0. Test-only DDL submissions are 3 for PG and 4 for MySQL,
plus two fixed CREATE/DROP submissions per engine, two fixture INSERTs and three
catalog/data SELECTs plus one version check per engine. SQLx connector setup is
separate from these explicit fixture submissions. No migration Tx or rollback
contract is tested/claimed.

PostgreSQL statements (no semicolon required):

```sql
ALTER TABLE "quent_dispatch_test"."migration_probe" ADD COLUMN "small" SMALLINT NULL
ALTER TABLE "quent_dispatch_test"."migration_probe" ADD COLUMN "normal" INTEGER NULL
ALTER TABLE "quent_dispatch_test"."migration_probe" ADD COLUMN "big" BIGINT NULL
```

MySQL statements:

```sql
ALTER TABLE `quent_dispatch_test`.`migration_probe` ADD COLUMN `small` SMALLINT NULL
ALTER TABLE `quent_dispatch_test`.`migration_probe` ADD COLUMN `normal` INTEGER NULL
ALTER TABLE `quent_dispatch_test`.`migration_probe` ADD COLUMN `big` BIGINT NULL
ALTER TABLE `quent_dispatch_test`.`migration_probe` ADD COLUMN `tiny` TINYINT NULL
```

Independent catalog reads observed original `a` followed by exactly the new names,
expected signed smallint/int/bigint/(tinyint), all nullable with catalog NULL
default. Original row `a=7` retained and all appended values read as SQL NULL.
Independent INSERT/SELECT retained i16::MIN, i32::MIN, i64::MIN and MySQL i8::MIN.
Catalog NULL does **not** establish original Absent versus explicit NULL syntax;
that ambiguity remains in the source contract. These seven successful ALTERs on
two ordinary disposable tables do not prove general DDL correctness, current live
source correspondence, arbitrary quoting/server versions, concurrent DDL behavior,
lock bounds, or multi-step rollback.

The unchanged `scripts/test-dispatch.sh` used only dedicated containers/roles:

- PG digest `sha256:16bc17c64a573ef34162af9298258d1aec548232985b33ed7b1eac33ba35c229`, observed 16.14, UTC, read committed default, UTF8/en_US.utf8.
- MySQL digest `sha256:2952e3be7807f06fc18de50b3ea1a632d5c70d63482ff7d7376fe3aa8999babf`, observed 8.4.8, +00:00, REPEATABLE-READ, utf8mb4/utf8mb4_bin; ONLY_FULL_GROUP_BY, STRICT_TRANS_TABLES, NO_ZERO_IN_DATE, NO_ZERO_DATE, ERROR_FOR_DIVISION_BY_ZERO, NO_ENGINE_SUBSTITUTION. New fixture table explicitly InnoDB; existing FOUND_ROWS regressions retained.

Only the fixture's limited quent_test role and dedicated quent_dispatch_test
namespace/catalog were used by new SQL tests. Existing hidden-role/alternate
catalog/trigger tests remain in full regression. Tables dropped, connections
closed; script removed its exact dedicated containers and anonymous volumes and
verified absence. No arbitrary DB or volume prune was used.

## Measured validation

WSL Ubuntu, Rust 1.94.0; sequential cargo/compiler/test runs:

- Default workspace: **113 passed**, 0 failed/ignored/filtered (95 retained +18).
- Full all-feature workspace on both dedicated DBs: **252 passed**, 0 failed/ignored/filtered (230 retained +22); cleanup absence verified.
- All-feature DB-free `--skip live_`: **234 passed**, 0 failed/ignored, 18 filtered. Three incidental-name exclusions individually selected and passed; the remaining 15 are live suites covered by full run. A filtered case is not counted as a pass.
- Shared compiler: **347 diagnostic-checked negatives +19 compiling controls** (319+18 retained); generated consumer **76 negatives +8 controls in six invocations** retained, including frozen four binaries and plain/bundle QCK1/QCC1/QCR1/QCC2/QCR2 regression.
- fmt, default/all-feature workspace build, all-target/all-feature `clippy -D warnings`, `RUSTDOCFLAGS=-D warnings` rustdoc and Core/default facade/CLI normal dependency trees passed. Core has zero dependencies; facade/CLI default normal trees contain only Core. Toolchain/edition/publish=false, optional features, all exact pins and Cargo.lock are unchanged.

The unchanged `scripts/check-features.sh --offline` passed all 262 cargo feature
checks and its three CLI test runs. The 30 facade +8 CLI test/Clippy combinations
are executed sequentially (76 commands); their final result and actual published
head/PR/hosted checks are recorded in the RelayWeft report after execution; job definitions are not proof of
hosted success. No local Windows program was launched. Native hosted Windows
checks are distinct from WSL evidence and never imply Windows ACL export support.
Initial development failures (Clippy collapsible-if/item ordering and test-only
SQLx 0.9 dynamic-SQL wrapper) were corrected before successful final checks; they
are not acceptance evidence. No failed/0-selected/filtered run substitutes for a
required test.

## Remaining unsupported and deferred acceptance

The [capability matrix](migration-plan-v1.md) is deliberately finite. No live
binding, preflight, phases, human apply, migration permit/driver/transaction
executor, arbitrary default/CHECK/expression SQL, namespace/table mutation,
index/FK/CHECK DDL, column drop/type/null/default DDL or constrained-table append
is implemented. Unknown source coverage and unsupported changes refuse the full
SQL sequence. Existing v1 manifests cannot supply missing named constraints or
live authorization. MySQL implicit commits and PostgreSQL transactional capability
are documented, not normalized into a general rollback promise. Lock/data-loss/
type-conversion limits, unknown catalogs, physical identity, external DDL races,
external delivery/general exactly-once, Windows ACL and all 24 whole-case runtime
acceptance markers remain unverified/deferred to their owning work.
