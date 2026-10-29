---
id: TM-005
title: "quire-contract-ir conformance test matrix"
type: TestMatrix
---
# quire-contract-ir conformance test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-018 | FR-018-AC-1 through FR-018-AC-3 | TC-018 | 🚧 inventory, diagnostic, dependency, coverage and canonical-byte comparisons implemented; removal of the digest expectation and file-regeneration gate awaits code and focused TC-018 controls (IR-1105) |
| FR-020 | FR-020-AC-1 through FR-020-AC-15 | TC-018 | 🚧 AC-3 through AC-15 retain their existing controls, including the eight decimal-string members, 2^53 revision and byte-offset bounds, decoder versus fixture-schema refusals, and independently authored canonical-byte expectations. AC-2's five-kind mismatch registry and the absence of a digest expectation await code and focused TC-018 controls (IR-1105). The corpus run and process-output controls remain required. |

| FR-020 | FR-020-AC-16 through FR-020-AC-19 | TC-018 | 🚧 implemented with focused controls (IR-941): authoritative root-only shared CLI dependency, caller-owned exits, unchanged bytes/argv, prefix-failing serialization and actual output-device failure; unchanged published files require PR-diff inspection. Independent review and final aggregate remain pending; existing AC-1 through AC-15 remain implemented. |

| FR-057 | FR-057-AC-1 through FR-057-AC-10 | Pending shared-owner verification allocation | 🚧 proposed/planned (IR-123): source/consumer/result association and adverse controls; shared owner concurrence and selected review set pending. Existing TC-018 producer checks are prerequisites only, not cross-repository acceptance evidence. |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-018 | Schema, corpus, diagnostics, canonical bytes, dependencies, and interfaces conform without digest expectations or file-regeneration gates | Integration | P0 | FR-018..FR-020 | 🚧 existing semantic, inventory and process checks remain; the revised five-kind mismatch registry, digest-field removal and no-regeneration-gate controls await code (IR-1105). FR-019-AC-8 through AC-13 retain independent boundary and signature checks |

