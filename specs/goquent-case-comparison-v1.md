# Goquent / Quent semantic case comparison (PR1)

Read-only reference: local `/home/murai/github/goquent`, clean worktree, HEAD
`c51e084d0d530cae3eb22ad2b0686e47c6b79f0c`; inspected
[contracts v3 section 4 case table at that commit](https://github.com/recoweft/goquent/blob/c51e084d0d530cae3eb22ad2b0686e47c6b79f0c/specs/goquent_ai_contracts_v3.md#4-shared-cases-documentary-specification-not-new-tests).
The last local commit touching this spec was
`bbc73c17cd47d1bafbcfe1864e4f149c506006cd`.
This is an actually read local committed specification snapshot, not verification
of remote current main or a fresh Go test/build result. The original Quent Issue1
reference `b60b6b73b0d0c997dc2bbd74971de4112a398606` is historical provenance;
this comparison does not silently reuse its measurements. Goquent is unmodified.

The Go table itself distinguishes current source-derived determination, required
determination and limits. Its historical descriptions must not be represented as
newly reproduced bugs or current-runtime guarantees. “Same intent” below describes
a documentary contract only. Every Go runtime comparison remains **unverified**;
no Go test/database/authorization/public wire comparison was executed. Quent's
finite API/test responsibilities are in the [24-case register](integration-acceptance-v1.md).
Equal case numbers do not imply equal semantics (e.g. Quent QT-C02 ↔ Go C03).

| Quent case | Actually inspected Go case(s) | Classification | Difference and responsibility |
| --- | --- | --- | --- |
| QT-C01 | C01 | 部分対応 | Pure plan intent overlaps; Quent explicit projection/Strict/private record; source-derived comparison only |
| QT-C02 | C03 | 同等（契約意図） | No-predicate writes blocked; Go compatibility/default soft-delete caveat is not adopted |
| QT-C03 | C04 | 意味相違あり | Both tenant-wide, not single row; historical Go medium vs Quent High + finite explicit provider |
| QT-C04 | C05/C10 | 部分対応 | Ranges/multiple IN/composite components need complete supported equality proof |
| QT-C05 | C06 | 同等（契約意図） | Every OR branch must establish scope; finite native/schema limits apply |
| QT-C06 | C07/C08 | 部分対応 | Strict unbound-branch refusal and outer AND; Go compatibility presence is not Quent authority |
| QT-C07 | C11 | 部分対応 | NOT/NULL precision overlaps; empty-IN normalization/native refusal do not imply general SQL support |
| QT-C08 | C09/C12/C19/C22/C23 | 意味相違あり | Trusted context and public claim nonpromotion; Quent has no raw/reason override |
| QT-C09 | C14 | 部分対応 | All candidate tenants checked; Quent general runtime batch remains unsupported |
| QT-C10 | — | 未検証 | No exact readonly/generated/three-state patch case mapped in inspected Go table; Quent patch execution unsupported |
| QT-C11 | C26/C27 | 部分対応 | Immutable DB context/borrowed inheritance overlap; custom Executor escape paths differ |
| QT-C12 | C19 | 意味相違あり | Quent private consumed full-content binding; inspected Go case describes mutable/untrusted plans |
| QT-C13 | — | 意味相違 | Rust linear carriers/Clone/serde compile-fail have no same-language Go counterpart established |
| QT-C14 | — | 未検証 | No matching expiry/cancel/request-bound provider case in inspected C01–C28 table; Go implementation not exercised |
| QT-C15 | C20 | 意味相違あり | Public secret exclusion intent overlaps; inspected Go historical source risks are not current runtime findings |
| QT-C16 | C16/C17/C18 | 部分対応 | Complete result/cardinality/owned rollback/caller responsibility; dialect paths differ |
| QT-C17 | C25 | 部分対応 | Durable identity/input/replay overlaps; no new exactly-once or external delivery guarantee |
| QT-C18 | C25 | 部分対応 | Quent explicit unknown/reconciliation/corrupt ledger boundaries; Go fault behavior not run |
| QT-C19 | C22/C23 | 部分対応 | Freshness verification overlaps; source/offline/live/domain authority distinctions differ |
| QT-C20 | C24 | 部分対応 | Source syntactic partial/unsupported intent; neither supplies whole-program proof |
| QT-C21 | C02 | 意味相違 | Go SELECT shape warnings are not Quent explicit non-ANALYZE estimate or request non-Tx SQL budget |
| QT-C22 | — | unsupported比較 | No matching migration case in inspected Go table; no cross-engine DDL equivalence established |
| QT-C23 | C21（Specのみ） | 部分対応 | Read-only Spec intent only; MCP transport/registration/authority equivalence unverified |
| QT-C24 | — | 未検証 | No exact bounded wire/version/duplicate/depth suite counterpart mapped; no wire compatibility promise |

No row establishes API/wire compatibility, matching SQL bytes, equal driver counts,
authenticated remote providers, external exactly-once or completed cross-language
runtime acceptance. Missing table counterparts are unverified comparisons, not
proof that Goquent has no such feature elsewhere. Neither repository's expectation
is weakened to copy the other's historical implementation.
