## Problem and resulting behavior

<!-- Describe the concrete change and current finite behavior for a new reviewer. -->

## Identity and scope

- Refs #<!-- Issue; do not use a closing keyword for an intermediate planned PR -->
- Management ID / planned PR / actual PR URL:
- Branch → base:
- Exact final source head (update after the last source commit):
- Dependency actual PRs: merged/base/merge SHA/actual base ancestry checked:
- Changed files / scope and support-refusal table links:
- Production/API/feature/output changes, or documentation-only:

## Contract and case evidence

<!-- Link existing positive/negative oracles and distinguish full cases from finite subcases. -->

| Case/subcase | Positive and required refusal | Command / source | Observed counts and outcome | Unverified / unsupported |
| --- | --- | --- | --- | --- |
| QT-C… | | | | |

- Common retained record/finalizer/gate/native binding correspondence:
- DB/business submissions vs auxiliary SQL vs provider calls/responses:
- Tx begin/commit/rollback attempts and ACKs; Unknown vs confirmed completion:
- Result consumption/expectations and independent observer evidence:
- Command same-ID replay/conflict/corrupt-ledger / external-delivery limits:
- Public allowlist/canary detection / authority nonrestoration:
- All24 full runtime_pending / original expectations / loader vocabulary retained:

## Verification at the exact source

| Command / feature set | OS/environment | Passed / failed / ignored / filtered / selected | Exit / limitation |
| --- | --- | --- | --- |
| | | | |

- New doc snippets/commands: compiled/run results, expected output and exits:
- Compiler refusals vs compiling controls; generated consumer invocations:
- Default/Core/mcp-only dependency trees and warning-denied docs/Clippy/fmt:
- Prior baseline references (label explicitly; never count as this execution):
- Failures/development failures, skips/filters/0selected and remaining checks:

## Disposable DB and OS evidence

- Each run's pinned engine versions/settings, role/namespace scope:
- Owned containers/anonymous-volume cleanup and independent absence, including failure:
- No production/DATABASE_URL/global setting changes:
- WSL vs hosted Ubuntu/Windows vs local Windows/live/ACL evidence:
- Approval/environment blocker: target, sandboxed/approved result, denial reason,
  remaining unverified checks (no credentials or raw sensitive logs):

## Final-head CI and remaining decisions

- Source head / Actions run IDs / tested merge ref if applicable:
- Expected jobset and all terminal conclusions (current CI:94 expanded jobs):
- Unsupported/unverified/publication blockers and separate decision/authorization:
- No automatic full-case completion, commercial safety or release declaration:

<!-- Public report boundary: do not paste SQL, values, identifiers from sensitive
sources, paths/snippets from user inputs, arbitrary errors, canary secrets, DSNs or
credentials. Use fixed classifications and bounded counts. Report to RelayWeft
separately when requested; this PR body is not the execution-specific report.
Merge/Issue close/release/tag/publish/version/license changes require their own
explicit scope; green CI or completing this template does not grant permission. -->
