# QT-AI-10 planned PR3: fixed command audit/outbox bundle

This application-only extension supports exactly business1 → audit INSERT1 →
outbox INSERT1 in one Quent-owned transaction. Business is the existing all-column
single INSERT or Complete Low UPDATE/checked optimistic version; every supported
expected count is exactly1. No companion-only/one-companion/variable-length/branching
or arbitrary transaction API is provided. Ordinary CRUD and plain command entry
contracts remain in [command-ledger-v1](command-ledger-v1.md). External delivery,
polling, delivery-ack updates, HTTP/email/message workers and callbacks are absent.
DB atomic rows are not network/external exactly-once.

## Durable input and API

Persist all original inputs, namespace/CommandId, stable record/event IDs, payload,
timestamps, ordering, mapping and original context before first submission. Quent
never generates an ID, random payload or current timestamp. All fields use the
existing bounded native Value vocabulary; no bytes/UUID/JSON/Any/serde/native-type
extension is introduced. Example timestamps/sequence are explicit signed BIGINT,
not a new temporal native family. Arbitrary SQL, closures, executor/driver callbacks
and provider write bypasses remain unavailable.

Core's non-Clone/Copy/serde `CommandCompanion::application_supplied` takes an
immutable INSERT Operation and five distinct role Columns: tenant, record/event ID,
main command ID, namespace, subject. All five belong to that exact table; tenant
must match the source native signed nonnullable type, other roles are nonnullable
Text. Record/event ID is explicit nonempty Text; main command ID and namespace must
be the exact lowercase hex of the durable 16-byte ID and 1..128 namespace bytes.
Subject must be the exact original authenticated application SubjectIdentity text.
The comparison is binary/native in memory, never database text collation. Other
columns (payload/time/order included) are ordinary fully explicit native values or
typed nullable NULL. The library does not infer role names from SQL/column spelling.

`CommandBundle::new(business, audit, outbox)` is private-field immutable input.
`current.prepare().prepare_command_bundle(&bundle, namespace_bytes, &id_bytes)`
is pure Core preparation, with no DB/provider/clock/permit. Public Core preparation
parts and sensitive getters cannot construct facade handles or execute a record.
With existing driver/runtime features:

```rust,ignore
let handle = session.validate_command_bundle(&current, &bundle, &namespace,
    persisted_id.clone(), &ledger, ttl)?;
let result = session.execute_command_bundle(&current, &bundle, handle).await?;
// Explicit reconciliation, freshly connected session and current authority:
let handle = fresh.validate_command_bundle(&current, &bundle, &namespace,
    persisted_id, &ledger, ttl)?;
let saved = fresh.lookup_command_bundle(&current, &bundle, handle).await?;
```

There is no replacement expectation parameter. Non-Clone/Copy/serde
`ValidatedCommandBundle` binds all three records to the same original session and
exclusive monotonic expiry. The future consumes it at construction, including
unpolled drop. Cancellation/error/timeout never restores a handle. Borrowed adapters
consume/refuse with Unsupported/NotStarted and perform no transaction completion.
`CommandResult` remains non-Clone/Copy/serde, with redacted Debug. Its existing
public ResultView contains only fixed state; trusted applications may explicitly
borrow business result and the ordered audit/outbox results through sensitive getters.

## Explicit finite tenant mapping (consulted option A)

The existing valid business DatabaseContext is the only trust source. Its exact
business Tenant policy/Column/native value match is still mandatory. Companion
role Columns form the explicit bundle-only additional Tenant declarations; they
are not installed as three incompatible Tenant policies into DatabaseContext.
Normal constructor, ordinary preparation and plain-command Column matching remain
unchanged. No public general tenant/context rebinding API or independent companion
context exists.

Internally the same policy engine uses the checked mapped Column with the original
native tenant value, checks explicit values unchanged or supplements only an absent
tenant, and retains the original full snapshot. Protected/Generated/PII/
RequiredFilter policy remains applicable; a conflicting policy on the mapped tenant
is refused. Full schema coverage/table/native/nullability/set/count checks follow
supplementation. Original omission versus explicit equal tenant stays distinct.
Alias, another schema/catalog, unknown/missing facts, nullable/wrong-native mapping,
duplicate roles or any reused business/audit/outbox/ledger table refuses before DB.
All three carry the same original context/session; mapping grants no ordinary
execution authority. This is an explicit application assertion of tenant relation,
not authentication of actor, live catalog, physical routing or external state.

## Installation, ownership and retained history

Install [PG](../fixtures/command-bundle/postgres.sql) or
[MySQL](../fixtures/command-bundle/mysql.sql) explicitly as the owner, outside command
traffic. The library does not create/alter/migrate/overwrite tables. Business,
ledger, audit and outbox must be distinct ordinary base tables in the same explicit
schema/catalog/physical DB/connection/owned Tx. Full correct schema, no hidden
columns/default/identity/serial/auto_increment/generated semantics, triggers/RLS/
external effects or incompatible concurrent DDL is required. MySQL ALL FOUR tables
must be InnoDB, strict modes and SQLx FOUND_ROWS retained. All-column coverage does
not authenticate defaults/engines/hidden effects. Failure to satisfy both policy
and full coverage means unsupported, including protected required columns.

Example companions have tenant BIGINT NOT NULL, record_id VARCHAR(128) NOT NULL,
command_id VARCHAR(32) NOT NULL, namespace VARCHAR(256) NOT NULL, subject
VARCHAR(128) NOT NULL, payload TEXT NULL, recorded_at/sequence BIGINT NOT NULL,
sequence >=0, primary key (tenant,record_id). Every value is explicitly bound;
record IDs reused by a different command may produce a constraint error and whole
rollback. In these examples payload is existing Text/typed NULL with max4096 UTF-8 bytes; other explicit data columns remain within the existing finite native Value domain. New native
JSON, arbitrary serde and opaque arbitrary bytes are unsupported. No default or
nullable omission synthesis exists. The fixture widths/constraints are explicit
physical application constraints, not a richer automatically verified Core type.

Use a dedicated application role: schema USAGE (PG), SELECT/INSERT on the exact
business/ledger/audit/outbox tables; only required business UPDATE columns and
ledger UPDATE(result). No audit/outbox UPDATE/DELETE, DDL/TRUNCATE, ledger history
rewrite or retention privileges. Separate installer/retention role owns backup,
storage growth, access control and retention. Do not delete ledger identities while
IDs can be reused/reconciled. Companion retention/deletion and any downstream
consumer are application responsibilities. Fixtures use disposable setup roles;
the test role is not evidence of production least privilege.

## Canonical/result versions and rollout

The lookup key remains **QCK1**, same authoritative ledger and namespace/target/
tenant/ID space as plain commands. QCC1/QCR1 plain bytes are unchanged. A plain
record never retroactively means audit/outbox existed and is never replayed with
new companion INSERTs. Plain ↔ bundle under the same key is IdentityConflict,
including lookup; no separate namespace/ledger is chosen to dodge it.

Bundle input is **QCC2**: the existing explicit typed length-delimited token grammar
with ledger placement, full ORIGINAL context and source tenant Column, fixed role
indices0/1/2 and expected counts1/1/1; mapped role Columns in order and source native
tenant relation; each exact ORIGINAL and final Operation, full SQL bytes, ordered
native args, caller-missing-WHERE and finite INSERT proof metadata. In this finite
subset the retained final Operation determines the remaining semantic metadata;
none is supplied by a separate mutable renderer/planner. Table/schema/context/
subject/tenant and all payload/ID/order values are included. No digest, SQL-only,
Debug, public JSON, sorting, coercion or textual collation substitutes for full bytes.
A structurally valid unequal input conflicts without overwrite/missing fallback.

Ledger version is signed SMALLINT1 for plain and2 for bundle; new readers accept
both and require header/version consistency. **QCR2** is exactly29 bytes: four-byte
header, one dialect tag, then three ordered u64 big-endian count1 values. Business,
audit and outbox results must ALL have no rows/count projection and exact1. QCR1
remains exactly13 bytes. Old readers reject version2/QCC2/QCR2; they cannot recover
bundle authority or silently treat it as plain. New readers keep QCK1/QCC1/QCR1
replay and never rewrite them.

Before enabling bundles, the installer must explicitly widen legacy ledger result
storage from13 to29 and replace version/result checks with version IN(1,2) and
length13 for version1 /29 for version2, preserving all existing binary key/input/
result bytes. See the full installation DDL; choose deployment-specific constraint
names/maintenance and backup separately. MySQL ALTER is outside owned command Tx
and may implicitly commit. No automatic migration, overwrite, old-data reinterpretation
or guarantee for concurrent old/new installation management is implemented. An old
installation that rejects version2/results must fail/rollback, never be bypassed.

| Budget / strict decode | Fixed limit |
| --- | --- |
| Entire bundle canonical, all payloads/metadata/operations combined | 1,048,576 bytes, refused before first DB send |
| Each original/final Operation | existing depth16/nodes1024/args1024/columns256/branch budgets |
| Each native Text / SQL byte token |4096 /131072 bytes |
| Key / namespace / ID |2048 /128 /16 bytes |
| Saved complete result |13 bytes plain,29 bytes bundle, exactly supported counts |
| Ledger read |LIMIT2, input prefix1,048,577 and result prefix30; extra/oversize rejected |
| Untrusted canonical grammar |depth32, tokens131072, known tags/native widths/types/UTF8/bool, exact boundaries, then EVERY byte equal |

The bundle has three fixed operations/results, so there is no unbounded record
count/depth or payload bag. Oversize never passes by truncation. Pending NULL result,
unknown version/header/dialect, malformed tokens/counts/truncation/trailing/duplicate
storage are fixed failures, never missing or automatic business fallback. Privileged
malicious history rewrite is not automatically detected/authenticated.

## Transaction, current authority and reconciliation

Fixed order: preflight all records/capabilities → begin → bounded lookup → (if
missing and execute) unique reservation → business send and complete result/exact1/
version check → audit send and complete result/exact1 → outbox send and complete
result/exact1 → save all three supported results → commit. Revalidate the complete
current original context, bundle/mapping, session, all record gates and expiry after
each awaited stage and immediately before sends/commit. Sample the clock AFTER
bounded comparisons; now==expiry refuses. Same private finalized records supply
inspection, native binding and actual dispatch; no appended SQL or alternate binder.

Version0 result is Conflict; ordinary0/multiple and companion0/multiple are fixed
ExpectationMismatch. Known validation/constraint/result/save failures only become
RolledBack after rollback ACK. Start/send/completion uncertainty quarantines and
reports OutcomeUnknown where a value can return; cancellation itself returns no
claim. Only commit ACK exposes FirstCommit/Committed. No automatic write retry or
new ID substitution exists, for either CRUD or command.

Saved-result replay/explicit lookup requires freshly supplied complete bundle and
current authority, compares the retained input/results strictly, and sends business0,
audit0,outbox0. A historical version write is never applied to the current version.
Changed authority cannot disclose results. None is only a successful bounded
missing read; it is neither rollback proof nor unconditional retry permission.
PR2 adopted race option A is retained: an independently started waiting follower
may reserve/continue after leader ACK rollback, for same or different input. Deleted
uncommitted inputs are not permanently remembered; uniqueness is conditional on
retained committed ledger/full comparison, not one send across all attempts.

Default Debug/Display/public JSON/tracing/errors/notices do not disclose SQL,
values/names/subject/tenant/IDs/payloads/canonical or arbitrary nested driver causes.
Fixed errors have sourceNone, quiet_driver and existing panic/local-details/
Windows UnsupportedOs/CI/MCP restrictions remain. Plan/prepare/compile/review/
codegen/OperationSpec/CLI/MCP remain DB/provider0 and have no write/permit API.
Ordinary High/non-supported INSERT protection remains unchanged. No external worker,
production authority/adapter/routing/DDL/retention certification is implied.

[Finite measurements and remaining limits](command-bundle-v1-evidence.md) distinguish
actual tests from unsupported and unverified behavior. All24 full runtime_pending
markers and original fixtures remain; this PR does not close Issue11.
