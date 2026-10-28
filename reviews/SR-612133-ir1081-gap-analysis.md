---
id: SR-612133
title: "IR-1081 PR 648 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1081; PR #648 head 318fe29ba05f8b9be2341dbd1aef1cbc3cc8559e; Cargo.toml, tests/tc019_determinism.rs; NFR-001-AC-2/3, TC-019"
review_set: subset
---

## Summary

Reviewed the NFR-001-AC-2/3 and TC-019 evidence path for PR #648. The PR introduces a local executable capture/comparator but does not yet run it on Linux, macOS and Windows in one qualified CI flow, and TC-019's Analysis binding remains with PLAT-1160.

## Verdict

CONDITIONAL. This partial harness slice is reviewable, but the two cross-platform criteria remain unbacked until genuine three-OS execution and a qualified evidence channel exist. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No CI job in this PR runs the capture on three supported operating systems or compares their artifacts, so direct cross-platform canonical byte equality required by NFR-001-AC-2 remains unverified. | NFR-001-AC-2; tests/tc019_determinism.rs:269 |
| FND-002 | medium | No genuine three-OS evidence or qualified TC-019 Analysis binding proves the complete repeated and cross-platform ordered diagnostic sequence required by NFR-001-AC-3. | NFR-001-AC-3; tests/tc019_determinism.rs:269 |

## Coverage

- NFR-001-AC-2 (examined): each fixed admitted corpus input and supported profile yielding canonical output must have directly equal canonical bytes on Linux, macOS and Windows when cross-platform CI is enabled.
- NFR-001-AC-3 (examined): each fixed rejected corpus input and profile must produce the same complete ordered `(code, path)` sequence in repeated runs and across supported operating systems.
- TC-019 (examined): Analysis evidence method remains unresolved under PLAT-1160; a source trace tag is not a substitute for that channel.
- All changed source and Cargo files were examined. No production behavior outside the named harness changed, and no applicable AssuranceProfile was found.
- Quoin matrix on the repository reports NFR-001-AC-2 and NFR-001-AC-3 `untagged`, each with zero binders and no run evidence. NFR-001-AC-1 is tagged by `tests/it/conformance.rs:136` but also has no run evidence in this matrix output. Plan completion: not assessed. Optional repository-wide semantic review was not requested; this pass inspected the scoped AC/test/code chain.
