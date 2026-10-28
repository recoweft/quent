# QT-AI-13 PR1: finite offline read-only MCP

This is a finite implementation of MCP **2025-11-25**, JSON-RPC 2.0,
UTF-8 newline-delimited **stdio only**, not full-standard/client-universal or Go
wire compatibility. No SDK is added. Official primary contracts:
[base](https://modelcontextprotocol.io/specification/2025-11-25/basic),
[lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle),
[stdio](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports),
[tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools),
[resources](https://modelcontextprotocol.io/specification/2025-11-25/server/resources),
[cancellation](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/cancellation).

The optional facade `mcp = [review]` and CLI `mcp = [dep:quent, quent/mcp]`
add no driver/runtime dependency. Exact pins, Rust 1.94.0, edition 2024, Core zero
normal dependencies, and default facade/CLI Core-only graphs remain unchanged.
Even with other features unified, handlers have no DB/session/provider/Tx/command/
migration/live/catalog/EXPLAIN API, connection configuration or callback argument.
No environment variable selects tools, a DSN, mapping, path, driver or live mode.
This is capability/source evidence, separate from observing zero loopback connects.

## Entry and immutable registration

```
cargo run -p quent-cli --no-default-features --features mcp --locked -- mcp --stdio
```

Exactly `mcp --stdio`, no positional paths or additional switches. Bad MCP grammar
exits 2 with empty stdout and fixed `QUENT_MCP_INVALID` stderr. With feature off,
the previous generic unsupported CLI route remains exit 1. There is no automatic
CLI forwarding to human apply, manifest verify live or any existing execute route.

`quent::mcp::Server::offline()` has no registration. The process reads no files,
remote URIs or configuration. Public context/schema report Missing/Unknown, with
Unverified freshness/runtime. Manifest status resource is Missing/Unverified.

`Server::application_registered(TypedModelContract, DatabaseContext)` takes BOTH
existing typed carriers from authenticated/authorized application Rust code. It
owns them immutably for exactly one stdio session/process. Independent current is
passed to the existing compiler, not copied from the mapping's private snapshot.
The application's assertions/authentication and subsequent refresh remain its
responsibility. Startup snapshots are not continuing live freshness, physical DB
identity or authorization. No mutable global registration or per-wire tenant,
subject, target, mapping or generation replacement exists. There is no registration
JSON, executor/provider/plugin callback, external Plan JSON receiver or public
operation/handle return API. A public summary cannot restore these private types.

A hosting application calls `serve_stdio()` then **exits the process promptly** with
`Termination::exit_code()`. It must own stdio and process panic policy exclusively;
the fixed process-wide panic hook discards payload/location/backtrace. The supplied
CLI does this. Do not reuse stdin/stdout or start another session after return.
Detached bounded workers can still await OS reads/writes; this is a process server,
not an embeddable multi-session background service or a general I/O callback API.

The ordinary executable has no authenticated startup registration distribution
mechanism: arbitrary-model typed compile success is **unsupported there**. Its
plan/compile calls return fixed `missing_application_context`; successful syntax
review is still Unknown and never a typed compile success. A test-only executable
registers synthetic typed carriers in Rust and runs the SAME library/transport/
handlers for successful stdio compilation. That fixture is not product registration
or evidence of arbitrary-model success in the ordinary CLI.

## Closed tools and resources

All names, descriptions, metadata, input schema values and URIs are constants.
There are four tools, each accepting exactly `{"source": string}` (UTF-8 bytes,
not a path, DSN, SQL fragment, callback or remote URI):

| Tool | Input / return | Ordinary CLI |
| --- | --- | --- |
| `quent_plan` | OperationSpec v1 → existing `SpecSource::compile` → retained `PlanView` | Missing registration refuses |
| `quent_compile` | Same common compiler/planner and output-only PlanView | Missing registration refuses |
| `quent_review` | With registration: existing `review::typed_spec`; otherwise existing syntax-only `review::operation_spec` | Syntax/literal checking only; runtime/binding unverified |
| `quent_manifest_status` | Existing `review::manifest` parser/public domain summary | Offline source only; independent domain freshness preserved |

Tool content is exactly one `type=text` containing existing public-view JSON,
`isError=false`; this is a finite analysis result, not execution acceptance. Fixed
precondition/input/unknown-tool failures use JSON-RPC errors, not apparently clean
or truncated tool content. There is no external Plan, Rust source, SARIF, baseline,
N+1, performance estimate, detailed artifact or execution tool. Parsing a manifest
is stateless: it never installs registration or changes the status resource.

| Resource URI | Finite public content |
| --- | --- |
| `quent://public/context` | Registration missing/application_asserted, freshness/runtime unverified, execution unavailable |
| `quent://public/schema` | Model count 0/1, registered column count 0..256, application_asserted/unknown coverage, freshness unverified |
| `quent://public/manifest-status` | Schema/policy/generated_code missing, live unverified; no retained manifest source |
| `quent://public/review-rules` | Fixed strict/read-only/offline/public-output/no-execution/no-permit/no-live-freshness codes |

Counts are structural application declarations, not verified schema facts. No
model/table/column/tenant/subject/target/alias/index/path/snippet/SQL/value/hash/free
reason/arbitrary error/opaque metadata enters these resources or tool output.
Details and fingerprints are never exported. Existing OperationSpec v1/typed
compiler rejection of writes, joins, aggregates, Raw, subqueries, hints, tenant
predicate/override, unknown kind/version/field and context mismatches is preserved.
No low-risk fallback or current-context manufacturing exists.

## Protocol support and lifecycle

| Protocol operation | Finite support |
| --- | --- |
| `initialize` | First initialization only; required string protocolVersion, empty capabilities object, clientInfo exactly string name/version |
| Version negotiation | Valid supported version echoes the constant; any other bounded string negotiates the implemented constant 2025-11-25. Client must disconnect if it does not support the answer |
| Capabilities | Server `resources:{}`, `tools:{}` only; client capabilities other than `{}` refuse |
| `notifications/initialized` | Ready only after initialize completes; no response; no reset/history clearing |
| `ping` | Empty/omitted params, including before initialization |
| `tools/list`, `resources/list` | One finite complete list, no pagination/cursor/dynamic discovery |
| `resources/templates/list` | Empty list; no URI template or arbitrary path loader |
| `resources/read`, `tools/call` | Only the exact allowlists after initialized; strict parameters |
| `notifications/cancelled` | Bounded ID lookup of current request only. Initialize cannot be cancelled. A matching active non-initialize result is discarded; no reply for the cancelled request. Completed/unknown/malformed cancellation is ignored |
| Other notifications | Ignored without reply, within frame/session budgets |
| Unknown methods/tools/URI/fields, unsupported capability | Fixed closed refusal; no generic dispatcher |
| EOF | Idle EOF graceful; partial unterminated frame invalid; already accepted work may finish before graceful exit |
| Shutdown | Close stdin; parent may terminate/kill/reap the child. No invented JSON-RPC shutdown/exit method |

In-flight means one accepted request until its result is consumed. Pipelined second
requests fail closed, rather than queueing arbitrary work. Notifications can be
observed while the worker is active. Cancellation cannot preempt synchronous pure
Rust parsing/planning: bounded work may finish, then its result is discarded. A
completion that wins the race may already have replied; no second response is sent.
No successful cancellation of DB work or hard CPU interruption is claimed.

## Public payload versus correlation ID

Request IDs are separate private carriers with no Debug/Display/serde. Valid
strings retain decoded UTF-8 exactly (no normalization/trim/hash), at most 128
bytes; integer tokens retain their exact integer value within
-9007199254740991..9007199254740991. The noncanonical negative-zero spelling `-0` is refused by the existing scanner.
Float/decimal/exponent spellings, null/bool/
array/object, out-of-range and invalid UTF-8 refuse before admission. String `"1"`
and integer `1` are distinct. The JSON encoding of escapes may change without
changing the decoded ID. No semantic filtering of SQL/tenant-looking IDs occurs.

For admitted requests only the response envelope's **top-level `/id`** carries the
same type/value, including fixed error responses. A secret placed in an ID will
therefore return in that ID: there is no claim of zero reflection of arbitrary
strings across ALL response bytes. Everywhere else—including nested `result.id`,
`error.data.id`, tool/resource content/metadata/schema, stderr/logs/errors—business
secrets and correlation IDs stay excluded. Cancellation requestId is lookup-only;
its optional reason is never logged. IDs are never operation identity/authority.
Use innocuous sequential integers in clients, not tenant/SQL/path secrets.

The test client parses the envelope independently, checks exact ID type/value,
then removes ONLY the top-level id before checking all other output. Detection
positive controls place canaries in result/nested id/error data/stderr; they must
be detected, not globally stripped by matching text or key name.

## Budgets, deadlines and termination

These are Quent profile limits, not universal MCP-required numbers.

| Boundary | Limit / behavior |
| --- | --- |
| Request frame | 65,536 bytes before LF, at most 8 KiB buffered read-ahead; no unbounded read-to-end |
| Source string | 4,096 decoded UTF-8 bytes, PLUS existing inner source grammar budgets |
| Pre-tree scanner | Existing OperationSpec scanner: depth 16 edges, 2,048 value nodes, 256 array entries, 32 unique object keys, key 128 bytes, every string 4,096 bytes |
| Parsing | Duplicate decoded keys refused BEFORE map construction; invalid UTF-8, multiple documents, floats, malformed/unknown JSON-RPC version refuse. No legacy receiver fallback |
| Response | 65,536 bytes INCLUDING LF and ID; excess closes with Budget BEFORE write, never truncates findings or returns a clean result |
| History | At most 256 accepted requests INCLUDING initialize; no reuse after success/failure/cancel/timeout; no LRU eviction |
| All frames | At most 512 requests+notifications, finite session history; unknown/cancel notifications consume this budget |
| In-flight/queues | One active request; input channel 1, worker-job channel 2 (initialized notification + request), completion/output/ack channels each 1 |
| Frame completion | 2s after first-byte event is observed; uncompleted input closes |
| Request processing | 2s after admission, coordinator does not block on the worker parser/planner |
| Output completion | 2s acknowledgement wait on bounded writer thread; partial public frame on OS I/O failure is not success |
| Idle/session | 5s inactivity, 60s total session, checked by coordinator; 2ms polling wait |

Reader parse and pure planner operate on bounded buffers off the coordinator.
Coordinator handles only bounded envelope data and finite response serialization.
These are elapsed-time checks when the coordinator is scheduled, not OS scheduling,
filesystem, hard CPU-preemption or universal I/O deadlines. A blocked stdin/stdout
worker is detached; the process-owning caller must exit after return. No worker
thread is allowed an unbounded queue, mutable registration or DB capability.

Malformed frames, invalid/out-of-profile IDs, reuse/history/in-flight overrun close
the session with an input-free termination classification, without echoing rejected
input or manufacturing an ID. Admitted handler failures preserve the admitted ID.
Notifications never receive replies. Normal MCP session exit has no stderr/banner.
`Termination`: Eof=0, Invalid=2, Budget=3, Timeout=6, Io=8. Panics retain fixed
`QUENT_PANIC` stderr, with no source/error payload. Fixed JSON-RPC error register:
-32600 invalid_state; -32601 method_not_found; -32602 invalid_params/input_refused;
-32002 resource_not_found; -32000 missing_application_context. No source chain.

## Finite acceptance and remaining unsupported

QT-C23 with C01/C08/C12/C15/C19/C20/C24: actual subprocess initialize/list/read/call,
registered PG/MySQL and ordinary unregistered CLI, same compiler/review public
results, strict refusals, manifest nonpromotion, independent public allowlist,
correlation and business canaries, invalid/budget/notification/EOF/time boundaries.
The test-only executable and client are `tests/mcp_stdio.rs` with a bounded channel
client and kill/reap guard, not a product command. Deterministic internal coordinator
controls separately hold completion to test in-flight cancellation, timeout and
second-request rejection; a response-budget control verifies no bytes are written.
These controls are not passed off as successful cancellation of OS/DB execution.

Compiler harness retains all 409 negatives/23 controls and adds 8 MCP carrier/API
negatives/1 compiling control (417/24). Existing publicview→context/request/permit/
ValidatedOperation restoration refusals remain; no new public authority input is
introduced. Generated consumer's 76 negatives/8 controls and regeneration remain.
MCP-only dependency trees and handler source establish unavailable DB/provider/Tx
capability. Real loopback detector with a connection-positive control separately
checks zero observed connections with an ambient DSN, including all-feature builds.
A synthetic recorder staying empty is not treated as DB/Tx instrumentation.

New MCP cases are entirely offline: new live DB cases are nonapplicable because no
DB/provider/Tx/catalog/EXPLAIN operation or driver change is introduced. Existing
full workspace regression still requires both pinned disposable DBs and cleanup.
Actual commands/results, initial failures, filters, OS/approvals, exact source head,
PR URL and terminal CI jobs belong to this execution's evidence/report, never older
PR evidence. All **24 full runtime_pending** entries remain unchanged. No safe
DB-autonomous execution, live freshness/authentication, Windows live DB, full MCP,
all-client interoperability, Go wire, HTTP, new runtime authorization, release or
publication claim follows from these finite tests.

Feature check script preserves the original 272 grid + 6 continuation checks and
adds 32 mcp grid + 2 continuation checks: **312 distinct explicit feature checks**
(304 grid + 8 continuation), with the existing three CLI test commands unchanged.
CI output-feature/CLI matrices retain their original 76 test/Clippy commands and
add 12 (four facade/two CLI combinations), **88 explicit commands per OS matrix
collection**. Actual expanded sets and terminal jobs must be checked for this head.
