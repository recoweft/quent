# QT-AI-09 planned PR2: scoped final execution gate

This is the current runtime extension to the [PR1 acquisition contract](authorization-contract-v1.md).
PR1 measurements and “not connected” statements describe its historical baseline.
`acquire_authorization` remains compatible and non-executing: its unbound receipt
has no conversion to the new executable candidate, permit or validated handle.
An arbitrary runtime session is never attached to a previously acquired receipt.

## Application trust and finite entries

The application authenticates HTTP/DB subjects and supplies authoritative current
snapshots, correct physical routing and trusted adapter registration. Labels,
constructors, nominal routing keys and adapter registration do not authenticate a
physical server, remote service or user. Adapter internal state/logging/panic/network
remain the application's responsibility. Library checks cannot observe arbitrary
external context-store changes unless the application supplies the changed snapshot.
Borrow immutability is distinct from external truth. Passing an obsolete snapshot
again cannot establish its currentness, and does not excuse omitting comparisons.

| Entry | PR2 support / refusal |
| --- | --- |
| Owned PG/MySQL `authorization_candidate` → `acquire_authorized` → `execute_authorized` | explicit request/session-bound finite UPDATE or physical DELETE; result Observed |
| Same candidate/acquisition → `execute_authorized_atomic` | one owned transaction; expectation 0..256; begin → common dispatch → full result inspection → expectation → commit/rollback |
| Borrowed PG/MySQL adapter candidate/acquisition → `execute_authorized` | fresh adapter identity tied to its existing Tx borrow; result Uncommitted; no owned commit/rollback or atomic guarantee |
| Borrowed `execute_authorized_atomic` | Unsupported / NotStarted; no begin/business send/completion |
| Ordinary CRUD / typed / generated / validated / atomic / compiled Spec | same common dispatcher; High/Destructive still refuse without the private permit; Low has no provider requirement or implicit callback |
| PR1 unbound acquisition / Plan / preparation / public views / reason / JSON | cannot convert to executable authority |
| Plan / prepare / compile / review / source / diff / baseline / manifest / codegen / MCP / CLI | no provider registration/permit/execution addition; existing explicit catalog API is independent |
| Required live freshness | `AuthorizationExecutionRequest::requiring_live_freshness()` refuses before provider; no fabricated Fresh or live capability |

Only Complete, application-asserted OrdinaryBaseTableWithoutEffects, Broad or
FiniteBound High UPDATE / High-or-Destructive physical DELETE under PR1 scopes
MultiRowUpdate / MultiRowDelete are eligible. Broad is an analysis classification,
not a row-count bound. All policy preparation/coverage/native/NULL rules remain.
MissingWhere/Blocked, missing/mismatched/updated tenant, unsupported native/schema,
PII/protected writes, Unknown/Partial, Raw/unknown families, INSERT, required freshness,
RETURNING, optimistic version, batch/upsert/join/new native families cannot be waived.
Raw has no production constructor; compile failure is its boundary, not a fake send.
Existing Low version/RETURNING atomic support is unchanged and cannot use this permit.
A High label, public summary, manifest Fresh, reason, review/SARIF/baseline or typed
success alone grants nothing. The existing dialect/native matrix remains authoritative.

## Ownership and one record

The application creates a non-Clone/Copy/serde `AuthorizationExecutionRequest` for
one authenticated runtime request. This is a local allocation identity, not remote
authentication. The exact session must create `ExecutableAuthorizationCandidate`
from current AuthorizationContext, request, original Operation, finite scope and TTL.
It prepares/finalizes once using Core, checks eligibility before provider, and
privately retains the real session and request identities **before** the callback.
The adapter can observe `is_runtime_bound`, scope/deadline and explicit sensitive
record/context/registration getters. It receives no connection, Tx, raw executor,
operation-replacement callback or authority-restoration API.

Acquisition consumes that candidate, verifies the original binding before provider,
uses PR1's per-invocation decision identity and content/expiry checks before/after
await, and moves the same prepared record into private `AuthorizedOperation`.
Dispatch consumes this non-Clone/Copy/serde permit at future creation, even if the
future is never polled. No failure returns it or issues an automatic retry/new ID.
The finalized SQL allocation, ordered native args and all metadata inspected by
the provider are borrowed by the same private native binder and business send.
No renderer/planner/binder/unsafe-executor bypass is added. PR1 unbound candidate
and acquisition types cannot be passed into these runtime APIs.

Current context must be supplied independently again to execute. The private
permit is never used as an implicit current snapshot. Original operation equality
uses full bounded content; retained final SQL/args/metadata cannot be replaced.
Comparison includes target/routing/catalog/dialect, subject/tenant, all settings/
policy/schema identities/generations/contents, exact immutable registration allocation
(and thus its full immutable identity/generation/config), request/session and scope.
Equal generations, fingerprints, public summaries or labels are insufficient.
External adapter configuration changes need a new registration; interior mutable
remote state cannot be authenticated by immutable registration comparison.

## Final gate and transaction order

The common dispatcher checks bound/current exact context and operation, permit
identity/eligibility or ordinary Strict refusal, then expiry. It prepares finite
native bindings from the retained record, repeats all current/operation/permit
checks, samples `Instant::now` **after** bounded comparison work and refuses at
`now >= deadline`. The next operation is the closed SQLx submission, with no
application await/callback between final check and send. TTL is (0,300s]. No hash
comparison, public summary comparison or caller clock replaces full checks.

Owned atomic execution checks before begin, then repeats in that same dispatcher
after the real begin await and after native binding. The owned Tx borrows that
exact connection and shares its private identity; there is no exposed independently
retargetable transaction. Borrowed adapters have fresh identities, including two
adapters over the same Tx, and cannot receive session/other-adapter permits.
Acquisition happens before owned begin; missing provider/deny/error/mismatch/expired
acquisition causes no begin/commit/rollback/business send. A borrowed adapter already
borrows an application-owned Tx when acquiring; provider failure leaves business
sends0 and performs no library commit/rollback on that existing Tx. Pre-begin execution
refusal is NotStarted. Refusal after begin explicitly rolls back; only acknowledged
rollback is RolledBack. Expectation mismatch is never excused by a permit. Supported
writes return no rows; their result is fully consumed before expectation/commit.
Authorized RETURNING/version decode is Unsupported, not claimed tested support.
Existing ordinary all-row decoding/version/RETURNING regressions remain required.

## Cancellation, errors and confidentiality

Deny/error/cooperative Cancelled/Timeout consume acquisition. A pending adapter
is not automatically timed out: the application may wrap the actual future in
its runtime timeout/cancellation mechanism. Dropping stops local polling, not
remote side effects. PR2 adds final expiry enforcement, not an OS-wide timeout
detector or external-service cancellation protocol. Unpolled and pre-send drop
consume the carrier with zero business sends; after owned begin, drop quarantines
the session and has no confirmed rollback result. After business send, timeout/drop
quarantines the session; the outcome is unknown. A future drop returns no error.
Driver loss returns OutcomeUnknown; only explicit acknowledged Tx completion is
reported as confirmed. No exactly-once guarantee follows from Rust ownership.
Provider calls, responses, business sends and Tx events are separate measurements.
In particular, an ordinary/direct autocommit write may already be applied after
submission even if the client never receives completion. Quarantine, socket drop,
no commit event and no rollback ACK do not prove rollback. An owned explicit Tx
without completion ACK is not promoted to RolledBack. Test-only server termination
before lock release and ordinary UPDATE response withholding distinguish these
finite observations; see [maintenance evidence](authorized-cancel-v1-evidence.md).

All new carriers have constant redacted Debug, private fields and no serialization,
Display export, public constructor/restoration or sensitive handle getter. Errors
are fixed enums with no arbitrary provider/driver source chain; their public view
uses existing `execution_risk_refused` for authorization failures. SQL, values,
names, subjects, tenants, configuration, reason and opaque decisions never enter
default output. Existing quiet native driver scope, panic boundary, Windows
UnsupportedOs detailed export and CI/MCP details refusal remain unchanged. Explicit
trusted adapter sensitive getters are the only new deliberate input exposure.
The library cannot sanitize independently implemented adapter logging/panics.

## Evidence boundaries

`execution/tests/authorized.rs` uses both production drivers/common dispatcher,
independent provider counter, final recorder and database observer. It observes
same SQL allocation/args/metadata at provider review and submission; tests direct,
owned commit/expectation rollback, borrowed Uncommitted and borrowed atomic refusal.
Controlled provider suspension/resume, independent changed current snapshot supply,
real elapsed expiry after provider/begin, exclusive deadline via private test clock,
and real server lock wait before runtime timeout/drop/disconnect distinguish zero
send from post-send unknown outcome. Product clock remains Instant::now; test-only
final barriers/clock never enter the public API or product cfg.
Compiler negatives have real diagnostic checks and compiling controls for carrier
ownership/private construction/restoration/unbound substitution. Existing public
canary, driver-notice positive control, CLI/FIFO, codegen consumers and all previous
DB suites remain regressions. Finite measured results/OS/CI/cleanup are recorded in
[the PR2 evidence](authorization-gate-v1-evidence.md), not inferred from test names.
All 24 full runtime_pending markers remain. External service authentication,
physical routing, concurrent DDL, general TLS/completion loss, Windows live DB and local
Windows SDK/ACL behavior are not established by these finite tests.
