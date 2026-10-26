# QT-AI-09 planned PR1: scoped authorization acquisition

This is an application-only facade contract, available without driver/runtime
features or new dependencies. Core remains dependency-free. PR1 adds explicit
acquisition, not an executable permit. The shared dispatcher, common Strict gate,
owned/borrowed transactions, validation, quarantine/no retry, expectation checks
and pre-commit decoding are unchanged. High/Destructive still refuse, including
after successful acquisition. PR2 owns final execution integration.

## Registration and trust

`AuthorizationContext::register` owns a complete immutable `DatabaseContext` and
an immutable registration: provider identity (1..128 ASCII identity bytes), nonzero
u64 configuration generation, full configuration bytes (0..4096), and a trusted
adapter instance. Configuration may be sensitive; no digest substitutes for it.
Subject must be present. The app authenticates the subject and implements/registers
the trusted adapter. Construction and nominal identity labels do not verify an
external service, credential, physical server, session or authentication protocol.

`Clone` copies the complete Core snapshot and shares the exact immutable
registration instance. `with_database` explicitly replaces the Core snapshot at
an exact equal database/routing/catalog/dialect tuple, retaining that registration.
It validates subject presence; Core validates the supplied snapshot composition.
Settings/policy/schema/subject/tenant are inherited from the explicitly supplied
Core snapshot, not from another DB or an ambient registry. Normal Core preparation
continues to inherit its own exact Core snapshot and has no provider registration.
The wrapper's candidate preparation uses that same Core preparation path once.
Separate registrations are distinct even with identical labels/generation/config.
Replacing a provider/config requires a new registration and new candidates.
No mutable global registry, singleton, ambient provider or tenant is introduced.
The trait requires Send+Sync, not purity: adapter interior state and its own
logging/network/side effects remain the trusted application's responsibility.

## Candidate eligibility and support

`candidate(&Operation, scope, ttl)` is DB/provider-free. It privately owns the
single immutable `PolicyPreparation`: original operation, final record, exact
context and selected facts. It is not a raw Plan or public-summary constructor.

| Scope/input | PR1 candidate/acquisition | Execution in PR1 |
| --- | --- | --- |
| MultiRowUpdate: UPDATE, Complete, Broad/FiniteBound, High | supported under complete application-asserted ordinary-table facts and successful policy preparation | existing Strict refusal |
| MultiRowDelete: physical DELETE, Complete, Broad/FiniteBound, High/Destructive | same conditional support; no soft-delete synthesis | existing Strict refusal |
| Low/Medium, Zero/Single, SELECT/COUNT | outside these scopes; existing normal paths need no provider | unchanged normal gate |
| MissingWhere, Unknown/Partial/NotApplicable, INSERT/InsertUnverified | Unsupported before provider | refused |
| Tenant missing/mismatch/update, PII, required filter, protected write, schema/policy/native/budget failure | fixed policy refusal before provider | refused |
| RETURNING, versioned authorization, batch/upsert/join/Raw/DDL/new native families | Unsupported/no production construction path; no capability inference | unchanged finite support/refusal |
| Required live freshness/physical identity not established | no such evidence supplied; PR1 acquisition cannot satisfy a runtime freshness requirement | cannot waive existing or future required checks |
| Missing provider | MissingProvider, no callback | High stays refused |

Complete Broad is a finite *analysis contract*, not a bounded affected-row count.
High alone is insufficient. Existing signed nonnullable scope proof semantics and
native encoders are not broadened. Aliases/noncanonical paths/unknown table effects
still refuse through preparation. Policies are applied before candidate eligibility;
external responses cannot remove structural blocks or change SQL/args/metadata.

## Exact binding and confidentiality

The retained preparation includes SQL bytes; ordered native values including
width/sign/NULL; all original and final table/path/column/alias/projection/write/
predicate AND/OR/NOT/operation branches/order/limit metadata; caller missing-WHERE;
complete target/routing/catalog/dialect; settings identity/generation/full policies;
schema identity/generation/all facts/keys/asserted semantics; subject and tenant
column/native value. It is never reconstructed from public counts, hashes or labels.
The wrapper additionally retains the exact registration instance, full identity,
configuration generation/content, requested enum scope and exclusive deadline.
The operation cannot be replaced after construction. Different values/branches/
metadata are different preparations even when public summaries match. There is no
operation replacement, scope substitution, expiry extension or serialization API.

`acquire_authorization(candidate, &current)` consumes the candidate at future
creation, checks complete snapshot plus registration identity, eligibility and
expiry before invoking the provider, and checks again after await. Each real-time
expiry sample follows all bounded content/decision comparisons at its boundary. Current is an
immutable borrowed snapshot throughout; safe API calls cannot change it in place.
A new derived context, even with the same generation and different contents, fails
comparison. The provider gets one `AuthorizationRequest` borrowing this candidate,
with a fresh private invocation allocation. `respond(self, AdapterResponse)`
consumes that request and associates the finite response with this invocation.
A saved decision from a canceled/other request fails pointer identity comparison.
No numeric ID/hash can be reused to manufacture an equal invocation. No automatic
resend or new-ID retry exists. AllowCandidate means only this acquisition response.

The sensitive request getters deliberately expose the full preparation/context and
full registration to the trusted in-process adapter; scope and monotonic deadline
are explicit. They expose no DB connection, executor, transaction or replacement
operation callback. SQL, values, schema names, routing, configuration, tenant and
subject are confidential. Debug for every carrier is constant redacted. There is
no source export, Display JSON, serde, FromStr or public-field restoration for
candidate/request/decision/acquisition. Only contexts are Clone; linear carriers
are neither Clone nor Copy. Existing protected sensitive export/Windows
UnsupportedOs, CI/MCP denial and fixed public serializers are unchanged. The
adapter decides what, if anything, to send to its own service: making complete
in-process data available proves no external service received/reviewed any of it.

PR1 has **no final runtime request/session binding**: invocation identity is the
acquisition call identity, not authentication of an HTTP request or DB session.
External current-context stores, physical routing changes, concurrent DDL, future
runtime session/capability/freshness checks are not observed by this immutable
borrow. PR2 must privately retain the finalized record and acquisition identity,
bind the actual request/session/current operation/context, and check all required
content and expiry after awaits and immediately before dispatch. Reusing an old
snapshot as “current” is not detected as an external truth change. An acquisition
has no getters/conversion/execute method; it cannot enter any existing dispatcher.

## Lifetime, time and failures

TTL is nonzero and at most 300 seconds, consistent with validated handles.
Time uses only this process's `std::time::Instant::now`, never caller wall-clock,
serialized time, provider claims or a default unknown time. `checked_add` refuses
overflow as InvalidTtl; `now >= deadline` is Expired (exclusive boundary). The
supported clock assumption is the standard process-local monotonic Instant;
unknown/unavailable remote clocks have no input API and are not validated. There
is no OS-wide response deadline or independent clock-health detection.

Finite errors: MissingProvider, MissingSubject, InvalidRegistration, InvalidTtl,
Policy(existing closed PolicyError), Unsupported, Mismatch, Expired, Denied,
ProviderError, Cancelled, Timeout. Display/Debug have only fixed classifications;
Error has no source. AdapterResponse has only AllowCandidate/Deny/Error/Cancelled/
Timeout, with no arbitrary error/reason/cause/string/credential/opaque payload.
No public authorization serializer, default logger, tracing emitter or panic with
input data is added. An independently implemented adapter must not emit sensitive
logs/panics; the library does not sanitize arbitrary application callbacks.

Deny/Error/Cancelled/Timeout returned by a provider are classified and consumed.
Timeout is cooperative reporting, not a runtime timeout detector. A provider that
never completes stays pending until its caller drops the future. Drop/abort/unpoll
consume the local candidate without returning a decision, retry handle or receipt;
Rust Drop cannot return Cancelled. Cancellation after polling does not undo an
external service side effect or prove service cancellation/exactly-once. No new
candidate/request IDs are automatically issued. Expiry after a provider returns
refuses even its allow response. No DB submission occurs in this module.

## Evidence and PR2 obligations

Tests in `authorization/tests.rs` use production APIs and cover both dialects, A/B exact
snapshots and immutable inheritance, subject/tenant/settings/policy changes,
identical-label separate providers, exact original/final input, scope mismatch,
closed response/error/source/redaction, missing provider, structural refusal,
unpolled/canceled polling counters and saved cross-request decision refusal,
before/at/after deadlines through the production private checker with deterministic
Instants; public acquisition covers expired entry and expiry across provider await.
Original/final input differences are retained-record comparisons, not a claimed
final runtime current-operation recheck (there is no replacement-operation input).
Invalid TTL and immutable-current borrow refusal are checked separately. The
tracing detector has an emitting positive control. Compiler tests have successful
acquisition/reply controls and check real diagnostic codes for linear ownership,
private constructors/fields, serde/FromStr/boolean/public-view restoration,
arbitrary errors, and dispatch substitution. They retain all previous controls.
Real PG/MySQL suites explicitly acquire from a counted fake, then observe zero
business sends for High/Destructive with the existing Strict gate. Ordinary Low
atomic execution succeeds and does not increment the provider counter.

These are fake in-process adapter contract tests, distinct from real DB/driver
regression and from **unimplemented external service/network authentication tests**.
Full runtime request/session changes during provider await, automatic timeout,
provider protocol cancellation, actual permit consumption at dispatch/Tx and
required runtime freshness remain PR2/unverified. No fictitious dispatch/Tx
records are created for acquisition. All 24 full runtime_pending markers remain.
C08/C11–15 gain limited evidence; C01/C02/C06/C10/C16/C19/C23 boundaries retain
existing refusal/normal paths. Command/migration/MCP/writeSpec/release/publish and
Issue36 schema/empty-source changes are outside this PR. Actual commands/counts,
CI/head/OS and disposable DB cleanup are recorded separately in the execution report.

[Measured local commands, counts, DB settings and limits](authorization-contract-v1-evidence.md).
