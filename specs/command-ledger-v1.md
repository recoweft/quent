Current PR3 extension: [fixed audit/outbox bundle](command-bundle-v1.md). The plain-command byte/API contracts below remain unchanged; PR3-pending statements describe the earlier PR1/PR2 baseline.

# QT-AI-10 planned PR1/PR2: finite durable commands

Application-only `PostgresSession` / `MysqlSession` APIs, under the existing
`postgres|mysql,runtime-tokio` features; no new dependency, feature, runtime, Core
serde or lockfile change. Preparation/Plan/review/codegen/OperationSpec/CLI/MCP
acquire no execution or provider rights. PR2 adds the finite version/concurrency/real
lost-reply cases below; PR3 audit/outbox remains pending. See [historical PR1 measurements](command-ledger-v1-evidence.md) and [PR2 measurements](command-concurrency-v1-evidence.md).
Historical “commands are future work” paragraphs in earlier PR documents describe
those baselines. Their ordinary CRUD, authorization, output and owned-result
boundaries remain current except for the explicit command entry below.

## Application protocol and support

The app must persist the original typed input, namespace, explicit 16-byte
`CommandId`, exact target/tenant/context and stable ledger installation BEFORE its
first submission. It supplies authentic current subject/tenant/settings/schema and
correct physical routing. Constructors do not authenticate those assertions.
Namespace bytes are trusted app configuration, 1..128 bytes, with binary equality;
IDs are application-chosen, not generated or replaced by Quent. Keep one authoritative
ledger installation per namespace/target; changing or deleting it cannot preserve
historical deduplication. Ledger placement is part of complete canonical input.

```rust,ignore
let namespace = quent::CommandNamespace::new(b"orders/v1".to_vec())?;
let id = quent::CommandId::new(persisted_id_bytes);
let ledger = quent::CommandLedger::application_supplied(ledger_table)?;
let h = session.validate_command(&current, &operation, &namespace,
    id.clone(), &ledger, std::time::Duration::from_secs(30))?;
let result = session.execute_command(&current, &operation, h).await?;
// Reconnect/restart: supply the SAME persisted original input and current authority.
let h = new_session.validate_command(&current, &operation, &namespace,
    id, &ledger, std::time::Duration::from_secs(30))?;
let saved = new_session.lookup_command(&current, &operation, h).await?;
```

| Input / outcome | Current PR1/PR2 boundary |
| --- | --- |
| Single INSERT | tenant policy and trusted signed tenant required; existing preparation fills only absent tenant; then exact full asserted column set/count/table/native type/nullability, no duplicates, explicit values or nullable typed NULL |
| Non-tenant omitted column, including nullable | Unsupported; no NULL/zero/empty/false/default synthesis |
| Default/identity/serial/auto_increment/generated-value dependence | Unsupported; app must assert their absence, not infer it from full-column coverage |
| Readonly/generated/PII/required-filter policy | existing policy refusal; full-column rule never waives protection; a table that cannot satisfy both rules is unsupported |
| UPDATE | existing Complete Low Zero/Single native/policy/key domain; tenant policy required; exact matched/affected count must be one before saving/commit |
| High/Destructive command authorization | refused; no authorized-command entry/permit restoration; ordinary High still needs its existing real request/session-bound permit |
| RETURNING/batch/upsert/join/Raw/DDL/new business native types | unsupported; no command result rows, generated IDs or driver callbacks |
| First successful execution | one business submission, affected/matched one, stored full supported result, acknowledged commit; `FirstCommit` |
| Same full key and exact complete input | current authority and all gates revalidated, stored result strictly checked; `Replayed`, business0 |
| Same key, different original/final input or context contents | `IdentityConflict`; no business send; no overwrite |
| Different namespace/tenant/target/catalog/dialect/ID | separate binary keys; lookup miss is never permission to return an unrelated result |
| Successful explicit lookup with no row | `None`; only this successful bounded read means missing, not proof that another transaction has finished |
| Pending NULL result, malformed/unknown/truncated/extra storage, DB error | ReservationUnresolved/LedgerInvalid/IdentityConflict/LedgerFailed; no business fallback |
| Reservation INSERT fails on retained winner | LedgerFailed, rollback attempted; no automatic replay/retry and no business send; reconcile explicitly in a fresh attempt |
| Borrowed Tx | command validation/execution/lookup Unsupported, NotStarted; external completion stays caller-owned |

`IdentityConflict` also safely rejects a structurally valid but nonmatching token
stream: invalid semantic shapes, duplicate/extra fields or wrong semantic types
never match a newly constructed valid canonical record. Public diagnostic views use
existing coarse conflict/decode/read-failed codes; the fixed typed ExecutionError
retains the precise classification. No arbitrary storage/driver cause is retained.

## PR2 exact expectation and optimistic version

`validate_command_expected(current, operation, namespace, id, ledger, expected, ttl)`
retains `ExpectedRows` privately in the consumed `ValidatedCommand`.
`validate_command` delegates with exact one; execute/lookup accept no replacement
expectation. The shared non-command atomic APIs still support their original
0..256 range. The answer to revision2 question
`02924154-ff13-425c-acf3-613a6349de25` fixes this finite table:

| Command | Supported expectation | Result before save/commit |
| --- | --- | --- |
| Full-column single INSERT, tenant supplementation only | exact1, no version | matched/affected1; unexpected0/multiple -> ExpectationMismatch |
| Existing Complete Low Zero/Single UPDATE | exact1 | matched1; 0/multiple -> ExpectationMismatch |
| Same UPDATE plus separate optimistic version | exact1 | matched1; 0 -> Conflict (including nonexistent key), multiple -> ExpectationMismatch |
| Any command with expected0 or2..256 | Unsupported during validation | begin/ledger/business/provider0, including an already retained ID; not IdentityConflict evidence |
| RETURNING, High command, batch/upsert/join/multi-operation Tx | Unsupported | existing refusal; no support expansion |

There is only one supported expectation, so no pair of different supported
expectations exists for an IdentityConflict positive case. Eligibility precedes
ledger access. Different **supported version values**, operation values/order,
original operation or current authority still require full canonical comparison.
Failed result checks roll back business and reservation together, and never save a
success result to justify resending. RolledBack requires rollback ACK.

Use existing `Draft::optimistic_version(column, expected)` on UPDATE. The original
predicate must independently prove the complete supported ordinary key; the separate
version predicate cannot supply key proof. Existing full tenant/schema/policy facts
and Complete Low inspection remain required. Signed nonnullable native widths:
Postgres I16/I32/I64, MySQL I8/I16/I32/I64. The same Core finalizer binds the
**checked native successor**, ordinary predicate/tenant guard and original expected
version once. Overflow (including native maximum), NULL, unsigned/bool/text,
nullable/wrong native schema, duplicate write/version assignment, tenant/readonly/
generated/PII version assignment refuse before business. App assertions cannot
verify physical custom domains or hidden DB constraints; these are not expanded.

Postgres matched counts and SQLx MySQL CLIENT_FOUND_ROWS are retained as distinct
result variants. An unchanged supported UPDATE has matched1 and may commit; MySQL
changed0 does not prove target absence or a version conflict. PR2's isolated native
control clears FOUND_ROWS only in its dedicated test handshake, observes changed0,
and compares with an unchanged production command's FOUND_ROWS1 and observer value.
There is no production connection-option/count reinterpretation switch.

QCK1/QCC1/QCR1, ledger column version1 and installation DDL are unchanged. Exact1
is a fixed protocol condition, sealed with the handle; canonical encoding for
pre-existing non-version operations remains byte-for-byte unchanged. Version already
has typed original/final IR and native arguments in QCC1. QCR1 stays exactly13 bytes,
matching dialect and count1, with no rows/count projection or arbitrary metadata.
No QCC2/QCR2/count0 support, automatic migration, reinterpretation or overwrite is
introduced. Unknown versions remain invalid. Initial experiments with exact0 were
superseded by the answer and are not final acceptance evidence.

## Canonical identity and current authority

`QCK1` stores the complete target database identity/routing key/catalog/dialect,
exact tenant column identity/native value, namespace bytes and ID bytes. Full binary
key equality, not hashes, text collation, SQL strings, Debug or JSON, selects a row.
`QCC1` stores ledger table identity, ORIGINAL operation (including omitted tenant),
full target/tenant/settings/policy/schema/subject snapshot, final operation/metadata,
final SQL bytes, ordered native arguments and private finite INSERT evidence.
Schema includes all facts in their retained order, native/nullability/context/key
kinds/components. Final metadata is determined by the retained final operation, including a version
condition and its checked successor, in this finite non-RETURNING subset; there is
no separately mutable metadata.
The same private immutable preparation supplies inspection, canonical encoding,
native binding and business submission. No second business renderer/binder exists.

The encoding has explicit numeric tags and big-endian lengths, native integer
width/sign, bool, UTF-8 text, byte sequences, SQL type and ordered containers.
Missing input, typed SQL NULL, empty text, zero and false have different encodings;
missing input is rejected before execution. Bytes and UTF-8 are distinct token
kinds; bytes are used for protocol SQL/key data, not a new supported business
`SqlType::Bytes`. Unsupported business types have no coercion/fallback. Predicate,
column and value order is retained. A supplied tenant and an omitted/filled tenant
remain different ORIGINAL inputs even if final SQL/arguments are identical.

Exact full content is compared, including same-generation different-content policy
or schema. Any identity/generation/content/subject change prevents replay under the
old input; there is no migration equivalence or disclosure across subjects. An app
that needs historical result access after changing authority/schema must design that
separately. It must not resupply obsolete snapshots to pretend they are current.
A new session may validate the durable ID/input again with identical authoritative
context; no old pointer/session/deadline/permit/request allocation is deserialized.
Low command execution has no provider path. Stored claims never mint a permit.

| Budget / validation | Limit |
| --- | --- |
| complete canonical input | 1,048,576 bytes; overflow refuses before reservation |
| key / namespace / ID | 2048 / 128 / 16 bytes |
| original/final IR | existing depth16, nodes1024, args1024, columns/order256 budgets; preparation branch budgets unchanged |
| canonical text / byte token | 4096 / 131072 bytes (rendered SQL uses byte token) |
| ledger read | fixed LIMIT2; per-row input prefix at most1,048,577 and result prefix14 bytes; two rows refuse; no unbounded payload read |
| untrusted token validation | depth32, total tokens131072, known tags/exact native widths, valid UTF-8/bool/type tags and exact length boundaries, then EVERY byte equal to current valid encoding |
| supported result | `QCR1`, exact13 bytes, exact dialect tag and u64 count exactly1; rows/count absent in this finite domain |

There is no arbitrary serde, unknown field bag, trailing-document parser, IR decoder,
handle factory or partial comparison. Unknown version/invalid token/truncation rejects;
well-formed but different full bytes conflict. Neither case becomes missing. Unknown
ledger version columns or result format/dialect/count reject. Stored data is untrusted;
structural checks do not authenticate malicious historical rewrites by privileged DB
operators. Retention/DDL/privilege integrity is an explicit installation responsibility.

## Ledger installation and physical assumptions

Explicit [PostgreSQL DDL](../fixtures/command-ledger/postgres.sql) and
[MySQL DDL](../fixtures/command-ledger/mysql.sql) include dedicated business examples.
Adapt only the validated two-component schema/table identifiers for the installation.
The library NEVER creates/alters tables. PostgreSQL: BYTEA primary key, BYTEA input,
nullable BYTEA result, signed SMALLINT version1, byte-length CHECKs. MySQL: binary
VARBINARY(2048) primary key (no prefix/hash index), LONGBLOB input, nullable
VARBINARY(13) result, signed SMALLINT version1, enforced CHECKs, InnoDB. No textual
collation equality is used. All application-written versions are exactly1; NULL
result is only an uncommitted reservation. Never commit one externally.

Business and ledger must be distinct ordinary tables in the SAME explicit schema
(PG) / catalog (MySQL), physical database, connection and owned transaction.
The app supplies complete accurate schema, no hidden columns/default/generated
semantics/triggers/RLS/external effects, no concurrent incompatible DDL and correct
routing. MySQL BOTH tables must use InnoDB; nontransactional engines and DDL implicit
commit cannot meet this contract. A nominal OrdinaryBaseTableWithoutEffects assertion
and all-column comparison are not live inspection of these facts. Sequence/default/
external-service effects are not made transactional. Fixture catalog observations
are test evidence, not a new production live-capability gate.

Use a dedicated app role, no global privileges and no DDL/DELETE/TRUNCATE/retention
rights. Example installation grants (run separately as owner, never inside a command):

```sql
-- PostgreSQL: create/authenticate quent_command_app separately.
GRANT USAGE ON SCHEMA quent_dispatch_test TO quent_command_app;
GRANT SELECT, INSERT ON quent_dispatch_test.command_items TO quent_command_app;
GRANT UPDATE(amount, flag, note) ON quent_dispatch_test.command_items TO quent_command_app;
GRANT SELECT, INSERT ON quent_dispatch_test.command_ledger TO quent_command_app;
GRANT UPDATE(result) ON quent_dispatch_test.command_ledger TO quent_command_app;
-- MySQL: create/authenticate this account with an appropriate host separately.
GRANT SELECT, INSERT ON quent_dispatch_test.command_items TO 'quent_command_app'@'app-host';
GRANT UPDATE(amount, flag, note) ON quent_dispatch_test.command_items TO 'quent_command_app'@'app-host';
GRANT SELECT, INSERT ON quent_dispatch_test.command_ledger TO 'quent_command_app'@'app-host';
GRANT UPDATE(result) ON quent_dispatch_test.command_ledger TO 'quent_command_app'@'app-host';
```

Keep installer/retention roles separate and revoke inherited public access as required
by the deployment. The disposable test role owns only dedicated fixture databases
and performs setup/observation; it does not prove deployment least privilege.
Applications own backup, access control, storage growth, retention and deletion.
Deleting ledger history while an ID can be reused can permit re-execution; no library
TTL/garbage collection or tombstone scheme is provided. Preserve IDs/results for the
application's entire reuse/reconciliation horizon and coordinate all replicas.

## Owned transaction, cancellation and unknown outcomes

Current snapshot/exact original operation/session/expiry/common risk and native
capability checks precede begin/reservation. Begin -> bounded lookup -> reserve
unique binary key -> SAME common business dispatcher -> full supported result
inspection/exact-one -> save result -> commit. After each awaited ledger/business
stage and immediately before each submission/commit, gates run again. Identifiers
use Core validation/quoting; internal ledger SQL is private, fixed, finite and typed
binary-bound. Its queries are not caller command SQL or a second business executor.
`&mut session` and immutable current/operation borrows prevent in-future replacement;
external state is authoritative only when the app supplies its changed snapshot.

Known business/ledger/result failures are RolledBack only after rollback ACK.
Begin/commit/rollback errors give OutcomeUnknown and quarantine; cancellation after
starting also quarantines without claiming an ACK. No handle is returned or cloned.
Unpolled/drop/timeout consume the carrier at future construction. A new durable-ID
lookup is a new validation, not reuse of a consumed linear operation. After an
ambiguous commit response, keep the SAME persisted ID/input and explicitly reconcile;
never replace it with a new ID or blindly retry ordinary CRUD. A lookup miss during
another transaction is not a proof of rollback. Historical PR1 measured one reservation winner/loser case and completion seams.
PR2 adds the finite server-observed interleavings and physical proxy cases below,
not general network-failure recovery or exactly-once. External delivery/callback
atomicity and audit/outbox remain outside this PR2.

Debug for IDs/namespaces/ledger/handles/canonical/result is fixed redacted; no Display,
serde restoration or arbitrary source cause. Sensitive canonical/result getters are
trusted application-only. Result public views contain only the existing fixed state;
kind/count observations do not authorize writes. SQLx notices/tracing remain inside
quiet_driver, statement/slow logging disabled. Existing local-details Windows
UnsupportedOs, CI/MCP refusal and panic/output boundaries are unchanged.

## PR2 finite race and real transport recovery

All attempts use distinct production sessions/owned transactions and fresh linear
handles. The following are finite outcomes under the installation assumptions:

| Controlled state | Subsequent attempt / reconciliation |
| --- | --- |
| Leader holds uncommitted reservation; lookup sees None | successful read-only observation only; neither rollback proof nor write retry/new-ID authority |
| Leader commits; waiting unique INSERT fails | follower LedgerFailed, business0; ACK rollback then explicit fresh session/current validation: same input Replayed, different input IdentityConflict |
| Leader ACK-rolls back; waiting unique INSERT succeeds | independently started follower may proceed directly to FirstCommit after all remaining gates; same OR different input |
| Leader sent business then expires after a server-observed row wait, ACK rollback | independent waiting follower may also send business and commit; two total business attempts, one retained committed ledger/result |
| Waiting attempt resumes expired or fails version/result checks | common gate / fixed result classification; rollback ACK required for RolledBack |
| Commit completion error, cancellation or uncertain outcome | quarantine/OutcomeUnknown (cancellation returns no value); no automatic retry or consumed-handle restoration |

A successful reservation INSERT belongs to that independent attempt. It is not
retrying a failed business operation or reviving an old handle. Rollback deletes
the leader's reservation/input with its Tx; durable comparison cannot remember
that vanished input forever. IdentityConflict requires a retained comparable record.
Do not promise one business submission across all attempts, external exactly-once,
or deduplication after deleting retained ledger history. No global mutex, separate
reservation transaction or coordinator is introduced. Failed-Tx/old-snapshot reads
are not used for reconciliation. Read-only lookup and explicit write attempts are
separate operations, both requiring new current authority and full input validation.

The dedicated test-only plaintext loopback proxy frames actual PG/MySQL protocol
messages on one connection, with bounded frame sizes/timeouts and no payload output.
It distinguishes COMMIT stopped before forwarding, COMMIT forwarded with a real
server acknowledgement consumed but not forwarded, and successful acknowledgement
forwarding. In the reply-loss case production SQLx receives a transport error:
OutcomeUnknown/quarantine, while an independent connection observes committed
business/ledger/result; a new session/current validation replays the saved result
with business0. This never reapplies the old version write to the current version.
The unchanged commit-success-then-forced-error seam remains separately labelled.
No global network/DNS/firewall setting or production force-error mechanism is added.
TLS, external networks, arbitrary packet timing, real physical faults and general
exactly-once are not verified by this finite plaintext experiment.

Storage counterexamples use deliberately invalid disposable installations and
actual stored binary values. Length-bounded reads/strict grammar/full equality/
result checks refuse unknown headers/version, invalid native widths/tags/bool/UTF8,
truncation, extra/duplicate semantic tokens, oversized input/result, count0/2,
wrong dialect, NULL fields and pending result. They never become missing or fall
through to business. Structurally valid unequal canonical data is IdentityConflict;
malformed data is LedgerInvalid; NULL pending result is ReservationUnresolved.
Decode failures after begin require explicit rollback; result payload remains13
bytes, input1MiB, key2048, and all original token/IR budgets apply. The restored
valid record is an actual decoder positive control, not a missing-row fallback.
Current authority changes still prevent old result disclosure. App-controlled
historical rewrites/retention/DDL/current stores/routing/adapter state and production
privileges are not automatically authenticated by these tests.
