---
id: SR-12991
title: integrity review of the TC-456 registered-code exemption matrix rows
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir; spec/core/matrix/TC-456-public-name-portability.md, spec/core/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-880. PR agent-ix/quire-contract-ir#474,, diff against origin/main. This review checks that the amended TC-456 expected results and status, the TC-456 and NFR-002 matrix rows, and the rollups that cite TC-456 agree with each other and with the tree. It also checks that `quire validate` is clean and that strict coverage does not regress.

## Examined scope

```yaml
scope:
- id: TC-456
  path: spec/core/matrix/TC-456-public-name-portability.md
  role: examined
  excerpt: "Planned until the NFR-002-AC-1 inventory test covers the amended scope. Its existing trace tag alone does not establish that coverage. TC-019 remains planned for NFR-001's separate determinism measurements."
- id: TC-456
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "| TC-456 |... | NFR-002-AC-1 | 🚧 planned:... it also lacks the exact STD-001 registry-derived exemption for registered `Std001Code` constant names and values, plus mutations for an unregistered `solver` constant and a removed registry row; adverse controls must pass through the inventory extractors |"
- id: NFR-002
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "| NFR-002 | vocabulary, schema and API inspection | TC-015, TC-016, TC-456 | AC-3/4 implemented; AC-1 public-name inventory, including the exact STD-001 registered `Std001Code` exemption, planned under TC-456 |"
- id: StR-002
  path: spec/core/matrix/tests.md
  role: context_only
  excerpt: "| StR-002 | FR-016 through FR-018, FR-020 | TC-017, TC-018, TC-019, TC-456 | 🚧 existing canonicalization and coverage behavior implemented;..."
- id: Core
  path: spec/tests.md
  role: context_only
  excerpt: "| Core | StR-001 through StR-004, FR-011, FR-012, FR-044, FR-045, NFR-001, NFR-002, NFR-003, STD-001 | [core](./core/matrix/tests.md) | 🚧..."
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. The TC-456 status still reads planned, and the TC-456 and NFR-002 matrix rows still read planned (🚧). The new "lacks" clause of the TC-456 row names exactly what the amended procedure adds: the derived exemption, the unregistered-`solver` mutation and the removed-row mutation. The tree confirms the gap: `tests/it/public_interface.rs::tc_456_public_schema_and_model_names_are_portable` neither parses STD-001 nor applies any exemption. No row claims coverage the code lacks, and the StR-002 and Core rollups, which this PR does not change, stay consistent. `quire validate --scope. 'spec/**/*.md'` exits 0 on the head, with output byte-identical to origin/main (module warnings only). `quire coverage --scope. --strict` output is byte-identical on origin/main and on the head: 40 unbacked rows existed before this PR, 0 contradicted statuses, exit 1 on both. Strict coverage therefore does not regress. The procedure-level defects are recorded under SR-12990.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The TC-015 row now says NFR-002-AC-3 credit is withheld "despite the existing trace tag". But neither TC-456, TC-015 nor IR-881's acceptance obliges anyone to reconcile that tag (`tests/it/identity.rs:128`). Only the AC-1 tag in `tests/it/public_interface.rs` is assigned to the TC-456 test PR. Until then `quire matrix` still reports NFR-002-AC-1 and NFR-002-AC-3 as `tagged`, identical to main, so the computed matrix contradicts the withheld credit. Strict coverage does not flag it (0 contradicted). | spec/core/matrix/tests.md:44; spec/core/matrix/TC-456-public-name-portability.md:74-77 |
| FND-003 | low | `spec/spec.md:62-64` still says TC-456 "verifies NFR-002-AC-1's public schema and model-name vocabulary". This PR now withholds that credit until IR-881 merges, so the overview overstates the claim. | spec/spec.md:62-64 |

Re-measured at: `quire validate --scope. 'spec/**/*.md' 'reviews/ir-880-pr474-*.md'` exits 0. `quire coverage --scope. --strict` and `quire matrix --scope.` output are byte-identical to origin/main. The TC-456, NFR-002, StR-002 and TC-015 rows all read planned (🚧) or withheld. No code or test files are in the diff.
